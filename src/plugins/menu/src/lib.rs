pub mod traits;

mod components;
mod events;
mod resources;
mod systems;
mod tools;
mod visualization;

#[cfg(debug_assertions)]
mod debug;

use crate::systems::{
	combos::visualize_invalid_skill::VisualizeInvalidSkill,
	dropdown::dropdown_skill_select_insert::DropdownSkillSelectInsert,
	update_panels::set_container_panels::SetContainerPanels,
};
use bevy::prelude::*;
use common::{
	resources::{key_map::KeyMap, language_server::LanguageServer, Shared},
	states::{game_state::GameState, menu_state::MenuState},
	tools::{inventory_key::InventoryKey, item_type::ItemType, slot_key::SlotKey},
	traits::{
		accessors::get::{GetField, GetFieldRef, Getter, GetterRef},
		handles_combo_menu::{EquipmentDescriptor, GetCombosOrdered, IsCompatible, NextKeys},
		handles_equipment::{
			Combo,
			HandlesEquipment,
			IsTimedOut,
			ItemAssets,
			ItemName,
			IterateQueue,
			PeekNext,
			WriteItem,
		},
		handles_graphics::{StaticRenderLayers, UiCamera},
		handles_inventory_menu::{
			Descriptions,
			GetDescriptor,
			InventoryDescriptor,
			QuickbarDescriptor,
			SkillExecution,
		},
		handles_load_tracking::{AssetsProgress, DependenciesProgress, HandlesLoadTracking},
		handles_player::HandlesPlayer,
		load_asset::Path,
		thread_safe::ThreadSafe,
	},
};
use components::{
	button_interaction::ButtonInteraction,
	combo_overview::ComboOverview,
	dropdown::Dropdown,
	inventory_panel::InventoryPanel,
	inventory_screen::InventoryScreen,
	key_select::{AppendSkill, KeySelect},
	key_select_dropdown_command::AppendSkillCommand,
	loading_screen::LoadingScreen,
	quickbar_panel::QuickbarPanel,
	skill_button::{DropdownItem, Horizontal, SkillButton, Vertical},
	start_game::StartGame,
	start_menu::StartMenu,
	start_menu_button::StartMenuButton,
	tooltip::{Tooltip, TooltipUI, TooltipUIControl, TooltipUiConfig},
	ui_overlay::UIOverlay,
};
use events::DropdownEvent;
use resources::equipment_info::EquipmentInfo;
use std::{hash::Hash, marker::PhantomData, time::Duration};
use systems::{
	adjust_global_z_index::adjust_global_z_index,
	combos::{
		dropdown_skill_select_click::DropdownSkillSelectClick,
		update_combos_view::UpdateComboOverview,
		update_combos_view_delete_skill::update_combos_view_delete_skill,
	},
	dad::{drag::drag, drop::drop},
	despawn::despawn,
	dropdown::{
		despawn_when_no_children_pressed::dropdown_despawn_when_no_children_pressed,
		detect_focus_change::dropdown_detect_focus_change,
		events::dropdown_events,
		select_successor_key::select_successor_key,
		spawn_focused::dropdown_spawn_focused,
		track_child_dropdowns::dropdown_track_child_dropdowns,
	},
	image_color::image_color,
	insert_key_code_text::insert_key_code_text,
	mouse_context::{prime::prime_mouse_context, set_ui::set_ui_mouse_context},
	on_release_set::OnReleaseSet,
	set_state_from_input::set_state_from_input,
	spawn::spawn,
	tooltip::tooltip,
	tooltip_visibility::tooltip_visibility,
	update_children::update_children,
	update_panels::{
		activity_colors_override::panel_activity_colors_override,
		colors::panel_colors,
		set_quickbar_icons::set_quickbar_icons,
		update_label_text::update_label_text,
	},
};
use traits::{insert_ui_content::InsertUiContent, GetLayout, GetRootNode, LoadUi};
use visualization::unusable::Unusable;

type SlotKeyMap = KeyMap<SlotKey, KeyCode>;

trait AddUI<TState> {
	fn add_ui<TComponent, TGraphics>(&mut self, on_state: TState) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + InsertUiContent,
		TGraphics: StaticRenderLayers + 'static;
}

impl<TState> AddUI<TState> for App
where
	TState: States + Copy,
{
	fn add_ui<TComponent, TGraphics>(&mut self, on_state: TState) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + InsertUiContent,
		TGraphics: StaticRenderLayers + 'static,
	{
		let spawn_component = (
			spawn::<TComponent, AssetServer, TGraphics>,
			update_children::<TComponent>,
		)
			.chain();

		self.add_systems(OnEnter(on_state), spawn_component)
			.add_systems(OnExit(on_state), despawn::<TComponent>)
			.add_systems(Update, update_children::<TComponent>)
	}
}

trait AddTooltip {
	fn add_tooltip<T>(&mut self) -> &mut Self
	where
		T: TooltipUiConfig + Clone + Sync + Send + 'static,
		Tooltip<T>: InsertUiContent;
}

impl AddTooltip for App {
	fn add_tooltip<T>(&mut self) -> &mut Self
	where
		T: TooltipUiConfig + Clone + Sync + Send + 'static,
		Tooltip<T>: InsertUiContent,
	{
		self.add_systems(
			Update,
			(
				tooltip::<T, TooltipUI<T>, TooltipUIControl, Window>,
				tooltip_visibility::<Real, T>,
			),
		)
	}
}

trait AddDropdown {
	fn add_dropdown<TItem>(&mut self) -> &mut Self
	where
		TItem: InsertUiContent + Sync + Send + 'static,
		Dropdown<TItem>: GetRootNode + GetLayout;
}

impl AddDropdown for App {
	fn add_dropdown<TItem>(&mut self) -> &mut Self
	where
		TItem: InsertUiContent + Sync + Send + 'static,
		Dropdown<TItem>: GetRootNode + GetLayout,
	{
		self.add_systems(
			Update,
			(
				dropdown_events::<TItem>,
				dropdown_track_child_dropdowns::<TItem>,
			),
		)
		.add_systems(
			Last,
			dropdown_detect_focus_change::<TItem>
				.pipe(dropdown_despawn_when_no_children_pressed::<TItem>)
				.pipe(dropdown_spawn_focused::<TItem>),
		)
	}
}

pub struct MenuPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TPlayers, TGraphics, TEquipment>
	MenuPlugin<(TLoading, TPlayers, TGraphics, TEquipment)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TPlayers: ThreadSafe + HandlesPlayer,
	TGraphics: ThreadSafe + UiCamera,
	TEquipment: ThreadSafe + HandlesEquipment,
{
	pub fn depends_on(_: &TLoading, _: &TPlayers, _: &TGraphics, _: &TEquipment) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TPlayers, TGraphics, TEquipment>
	MenuPlugin<(TLoading, TPlayers, TGraphics, TEquipment)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TPlayers: ThreadSafe + HandlesPlayer,
	TGraphics: ThreadSafe + UiCamera,
	TEquipment: ThreadSafe + HandlesEquipment,
{
	fn resources(&self, app: &mut App) {
		app.init_resource::<Shared<Path, Handle<Image>>>()
			.insert_resource(TooltipUIControl {
				tooltip_delay: Duration::from_millis(500),
			});
	}

	fn events(&self, app: &mut App) {
		app.add_event::<DropdownEvent>();
	}

	fn state_control(&self, app: &mut App) {
		app.add_systems(Update, set_state_from_input::<GameState>);
	}

	fn start_menu(&self, app: &mut App) {
		let start_menu = GameState::StartMenu;
		let new_game = GameState::NewGame;

		app.add_ui::<StartMenu, TGraphics::TUiCamera>(start_menu)
			.add_systems(Update, panel_colors::<StartMenuButton>)
			.add_systems(Update, StartGame::on_release_set(new_game));
	}

	fn loading_screen(&self, app: &mut App) {
		let load_assets = TLoading::processing_state::<AssetsProgress>();
		let load_dependencies = TLoading::processing_state::<DependenciesProgress>();

		app.add_ui::<LoadingScreen<AssetsProgress>, TGraphics::TUiCamera>(load_assets)
			.add_ui::<LoadingScreen<DependenciesProgress>, TGraphics::TUiCamera>(load_dependencies);
	}

	fn ui_overlay(&self, app: &mut App) {
		Self::register_quickbar_menu(app, Self::get_quickbar_descriptors);

		let play = GameState::Play;

		app.add_ui::<UIOverlay, TGraphics::TUiCamera>(play)
			.add_systems(
				Update,
				(
					update_label_text::<SlotKeyMap, LanguageServer, QuickbarPanel>,
					panel_colors::<QuickbarPanel>,
				)
					.run_if(in_state(play)),
			)
			.add_systems(
				Update,
				(
					set_ui_mouse_context,
					prime_mouse_context::<SlotKeyMap, QuickbarPanel>,
				),
			);
	}

	// BEGIN: Temporary proof of concept
	// This system needs to be moved into the future menu plugin interface
	fn register_combo_menu<TUpdateCombos, TEquipmentInfo, M1, M2>(
		app: &mut App,
		get_equipment_info: impl IntoSystem<(), Option<TEquipmentInfo>, M1>,
		update_combos: TUpdateCombos,
	) where
		TUpdateCombos: IntoSystem<In<Combo<Option<TEquipment::TSkill>>>, (), M2> + Copy,
		TEquipmentInfo: IsCompatible<TEquipment::TSkill>
			+ NextKeys
			+ GetCombosOrdered<TEquipment::TSkill>
			+ ThreadSafe,
	{
		let combo_overview = GameState::IngameMenu(MenuState::ComboOverview);

		app.add_ui::<ComboOverview<TEquipment::TSkill>, TGraphics::TUiCamera>(
			GameState::IngameMenu(MenuState::ComboOverview),
		)
		.add_dropdown::<SkillButton<DropdownItem<Vertical>, TEquipment::TSkill>>()
		.add_dropdown::<SkillButton<DropdownItem<Horizontal>, TEquipment::TSkill>>()
		.add_systems(
			Update,
			(
				get_equipment_info.pipe(EquipmentInfo::update),
				select_successor_key::<EquipmentInfo<TEquipmentInfo>>,
				Vertical::dropdown_skill_select_insert::<
					TEquipment::TSkill,
					EquipmentInfo<TEquipmentInfo>,
				>,
				Horizontal::dropdown_skill_select_insert::<
					TEquipment::TSkill,
					EquipmentInfo<TEquipmentInfo>,
				>,
				Vertical::dropdown_skill_select_click::<TEquipment::TSkill>.pipe(update_combos),
				Horizontal::dropdown_skill_select_click::<TEquipment::TSkill>.pipe(update_combos),
				update_combos_view_delete_skill::<TEquipment::TSkill>.pipe(update_combos),
				ComboOverview::<TEquipment::TSkill>::update_combos_overview::<
					TEquipment::TSkill,
					EquipmentInfo<TEquipmentInfo>,
				>,
				Unusable::visualize_invalid_skill::<
					TEquipment::TSkill,
					EquipmentInfo<TEquipmentInfo>,
				>,
			)
				.chain()
				.run_if(in_state(combo_overview)),
		);
	}

	// This system needs to be moved into the future menu plugin interface
	fn register_item_menu<TInventory, TSlots, M1, M2>(
		app: &mut App,
		get_inventor_labels: impl IntoSystem<(), Option<TInventory>, M1>,
		get_slot_labels: impl IntoSystem<(), Option<TSlots>, M2>,
	) where
		TInventory: GetDescriptor<InventoryKey> + ThreadSafe,
		TInventory::TItem: Getter<Name>,
		TSlots: GetDescriptor<SlotKey> + ThreadSafe,
		TSlots::TItem: Getter<Name>,
	{
		let inventory = GameState::IngameMenu(MenuState::Inventory);

		app.add_systems(
			Update,
			(
				get_inventor_labels.pipe(EquipmentInfo::update),
				get_slot_labels.pipe(EquipmentInfo::update),
				InventoryPanel::set_container_panels::<InventoryKey, EquipmentInfo<TInventory>>,
				InventoryPanel::set_container_panels::<SlotKey, EquipmentInfo<TSlots>>,
				panel_colors::<InventoryPanel>,
				drag::<TEquipment::TSwap, InventoryKey>,
				drag::<TEquipment::TSwap, SlotKey>,
				drop::<TEquipment::TSwap, InventoryKey, InventoryKey>,
				drop::<TEquipment::TSwap, InventoryKey, SlotKey>,
				drop::<TEquipment::TSwap, SlotKey, SlotKey>,
				drop::<TEquipment::TSwap, SlotKey, InventoryKey>,
			)
				.chain()
				.run_if(in_state(inventory)),
		);
	}

	// This system needs to be moved into the future menu plugin interface
	fn register_quickbar_menu<TCache, M>(
		app: &mut App,
		get_quickbar_cache: impl IntoSystem<(), Option<TCache>, M>,
	) where
		TCache: GetDescriptor<SlotKey, TItem = QuickbarDescriptor> + ThreadSafe,
	{
		let play = GameState::Play;

		app.add_systems(
			Update,
			(
				get_quickbar_cache.pipe(EquipmentInfo::update),
				set_quickbar_icons::<EquipmentInfo<TCache>>,
				panel_activity_colors_override::<SlotKeyMap, QuickbarPanel, EquipmentInfo<TCache>>,
			)
				.chain()
				.run_if(in_state(play)),
		);
	}

	// This system needs to move to the skills plugin
	#[allow(clippy::type_complexity)]
	fn get_equipment_info(
		slots: Query<(Ref<TEquipment::TSlots>, Ref<TEquipment::TCombos>), With<TPlayers::TPlayer>>,
		items: Res<Assets<TEquipment::TItem>>,
	) -> Option<EquipmentDescriptor<TEquipment::TSkill>> {
		let (slots, combos) = slots.get_single().ok()?;

		if !slots.is_changed() && !combos.is_changed() {
			return None;
		}

		let item_types = slots
			.item_assets()
			.filter_map(|(key, handle)| {
				let handle = handle.as_ref()?;
				let item = items.get(handle)?;
				Some((key, ItemType::get_field(item)))
			})
			.collect();

		let combo_keys = combos
			.combos_ordered()
			.iter()
			.flat_map(|combo| combo.iter())
			.map(|(combo_keys, ..)| combo_keys.clone())
			.collect();
		let combos = combos.combos_ordered();

		Some(EquipmentDescriptor {
			item_types,
			combo_keys,
			combos,
		})
	}

	// This system needs to move to the skills plugin
	fn update_combos(
		In(updated_combos): In<Combo<Option<TEquipment::TSkill>>>,
		mut combos: Query<&mut TEquipment::TCombos, With<TPlayers::TPlayer>>,
	) {
		let Ok(mut combo_component) = combos.get_single_mut() else {
			return;
		};

		for (combo_keys, skill) in updated_combos {
			combo_component.write_item(&combo_keys, skill);
		}
	}

	// This system needs to move to the skills plugin
	fn get_descriptors<TContainer, TSkill>(
		containers: Query<&TContainer, (With<TPlayers::TPlayer>, Changed<TContainer>)>,
		items: Res<Assets<TContainer::TItem>>,
		skills: Res<Assets<TSkill>>,
	) -> Option<Descriptions<TContainer::TKey, InventoryDescriptor>>
	where
		TContainer: ItemAssets + Component,
		TContainer::TKey: Eq + Hash + Copy,
		TContainer::TItem: Asset + Getter<ItemName> + GetterRef<Option<Handle<TSkill>>>,
		TSkill: Asset + GetterRef<Option<Handle<Image>>>,
	{
		let container = containers.get_single().ok()?;
		let map = container
			.item_assets()
			.filter_map(|(key, handle)| {
				let handle = handle.as_ref()?;
				let item = items.get(handle)?;
				let skill_handle = Option::<Handle<TSkill>>::get_field_ref(item).as_ref();
				let image = skill_handle
					.and_then(|handle| skills.get(handle))
					.and_then(|skill| Option::<Handle<Image>>::get_field_ref(skill).as_ref());
				let ItemName(name) = ItemName::get_field(item);

				Some((
					key,
					InventoryDescriptor {
						name,
						icon: image.cloned(),
					},
				))
			})
			.collect();

		Some(Descriptions(map))
	}

	// This system needs to move to the skills plugin
	#[allow(clippy::type_complexity)]
	fn get_quickbar_descriptors(
		queues: Query<
			(
				Ref<TEquipment::TSlots>,
				Ref<TEquipment::TQueue>,
				Ref<TEquipment::TCombos>,
				Option<&TEquipment::TCombosTimeOut>,
			),
			With<TPlayers::TPlayer>,
		>,
		items: Res<Assets<TEquipment::TItem>>,
		skills: Res<Assets<TEquipment::TSkill>>,
	) -> Option<Descriptions<SlotKey, QuickbarDescriptor>> {
		let (slots, queue, combos, combos_time_out) = queues.get_single().ok()?;

		if !Self::any_true(&[slots.is_changed(), queue.is_changed(), combos.is_changed()]) {
			return None;
		}

		let mut queue = queue.iterate();
		let active = queue.next();
		let queued_keys = queue.map(SlotKey::get_field).collect::<Vec<_>>();
		let combo_active = combos_time_out
			.map(|time_out| !time_out.is_timed_out())
			.unwrap_or(true);

		let map = slots
			.item_assets()
			.filter_map(|(key, handle)| {
				let handle = handle.as_ref()?;
				let item = items.get(handle)?;
				let handle = Option::<Handle<TEquipment::TSkill>>::get_field_ref(item).as_ref()?;
				let skill = skills.get(handle)?;
				let item_type = ItemType::get_field(item);
				let active = active.and_then(|skill| {
					let active_key = SlotKey::get_field(skill);
					if active_key != key {
						return None;
					}

					Some((
						Option::<Handle<Image>>::get_field_ref(skill).clone(),
						Name::get_field(skill).to_string(),
					))
				});

				let execution = match active {
					Some(_) => SkillExecution::Active,
					None if queued_keys.contains(&key) => SkillExecution::Queued,
					_ => SkillExecution::None,
				};

				let (icon, name) = match (active, combos.peek_next(&key, &item_type)) {
					(Some((active_icon, active_name)), _) => (active_icon, active_name),
					(_, Some(next_combo_skill)) if combo_active => (
						Option::<Handle<Image>>::get_field_ref(&next_combo_skill).clone(),
						Name::get_field(&next_combo_skill).to_string(),
					),
					_ => (
						Option::<Handle<Image>>::get_field_ref(skill).clone(),
						Name::get_field(skill).to_string(),
					),
				};

				Some((
					key,
					QuickbarDescriptor {
						name,
						icon,
						execution,
					},
				))
			})
			.collect();

		Some(Descriptions(map))
	}

	fn any_true(values: &[bool]) -> bool {
		values.contains(&true)
	}
	// END: Temporary proof of concept

	fn combo_overview(&self, app: &mut App) {
		Self::register_combo_menu(app, Self::get_equipment_info, Self::update_combos);

		app.add_dropdown::<KeySelect<AppendSkill>>()
			.add_tooltip::<Name>();
	}

	fn inventory_screen(&self, app: &mut App) {
		Self::register_item_menu(
			app,
			Self::get_descriptors::<TEquipment::TInventory, TEquipment::TSkill>,
			Self::get_descriptors::<TEquipment::TSlots, TEquipment::TSkill>,
		);

		let inventory = GameState::IngameMenu(MenuState::Inventory);

		app.add_ui::<InventoryScreen, TGraphics::TUiCamera>(inventory);
	}

	fn general_systems(&self, app: &mut App) {
		app.add_tooltip::<&'static str>()
			.add_systems(Update, image_color)
			.add_systems(Update, adjust_global_z_index)
			.add_systems(
				Update,
				insert_key_code_text::<SlotKey, SlotKeyMap, LanguageServer>,
			)
			.add_systems(Last, ButtonInteraction::system);
	}
}

impl<TLoading, TPlayers, TGraphics, TEquipment> Plugin
	for MenuPlugin<(TLoading, TPlayers, TGraphics, TEquipment)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TPlayers: ThreadSafe + HandlesPlayer,
	TGraphics: ThreadSafe + UiCamera,
	TEquipment: ThreadSafe + HandlesEquipment,
{
	fn build(&self, app: &mut App) {
		self.resources(app);
		self.events(app);
		self.state_control(app);
		self.start_menu(app);
		self.loading_screen(app);
		self.ui_overlay(app);
		self.combo_overview(app);
		self.inventory_screen(app);
		self.general_systems(app);

		#[cfg(debug_assertions)]
		{
			debug::setup_run_time_display::<TGraphics::TUiCamera>(app);
			debug::setup_dropdown_test(app);
		}
	}
}
