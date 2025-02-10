pub mod traits;

mod components;
mod events;
mod systems;
mod tools;
mod visualization;

#[cfg(debug_assertions)]
mod debug;

use crate::systems::{
	combos::visualize_invalid_skill::VisualizeInvalidSkill,
	dropdown::dropdown_skill_select_insert::DropdownSkillSelectInsert,
	items::swap::inventory_items::SwapInventoryItems,
	update_panels::container_states::SetContainerPanels,
};
use bevy::prelude::*;
use common::{
	resources::{key_map::KeyMap, language_server::LanguageServer, Shared},
	states::{game_state::GameState, menu_state::MenuState},
	systems::log::log_many,
	tools::{inventory_key::InventoryKey, slot_key::SlotKey},
	traits::{
		handles_equipment::{HandlesEquipment, SkillDescription},
		handles_graphics::{StaticRenderLayers, UiCamera},
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
use std::{marker::PhantomData, time::Duration};
use systems::{
	adjust_global_z_index::adjust_global_z_index,
	combos::{
		dropdown_skill_select_click::DropdownSkillSelectClick,
		update_combos_view::UpdateComboOverview,
		update_combos_view_delete_skill::update_combos_view_delete_skill,
	},
	conditions::{added::added, changed::changed},
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
	items::swap::equipped_items::swap_equipped_items,
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
		get_quickbar_icons::get_quickbar_icons,
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
				dropdown_detect_focus_change::<TItem>
					.pipe(dropdown_despawn_when_no_children_pressed::<TItem>)
					.pipe(dropdown_spawn_focused::<TItem>),
			),
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
		let play = GameState::Play;

		app.add_ui::<UIOverlay, TGraphics::TUiCamera>(play)
			.add_systems(
				Update,
				(
					get_quickbar_icons::<
						TPlayers::TPlayer,
						TEquipment::TSlots,
						TEquipment::TQueue,
						TEquipment::TCombos,
						TEquipment::TCombosTimeOut,
					>
						.pipe(set_quickbar_icons),
					update_label_text::<SlotKeyMap, LanguageServer, QuickbarPanel>,
					panel_colors::<QuickbarPanel>,
					panel_activity_colors_override::<
						TPlayers::TPlayer,
						SlotKeyMap,
						TEquipment::TQueue,
						QuickbarPanel,
					>,
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

	fn combo_overview(&self, app: &mut App) {
		let combo_overview = GameState::IngameMenu(MenuState::ComboOverview);

		app.add_ui::<ComboOverview<TEquipment::TSkill>, TGraphics::TUiCamera>(combo_overview)
			.add_dropdown::<SkillButton<DropdownItem<Vertical>, TEquipment::TSkill>>()
			.add_dropdown::<SkillButton<DropdownItem<Horizontal>, TEquipment::TSkill>>()
			.add_dropdown::<KeySelect<AppendSkill>>()
			.add_tooltip::<SkillDescription>()
			.add_systems(
				Update,
				ComboOverview::<TEquipment::TSkill>::update_combos_overview::<
					TPlayers::TPlayer,
					TEquipment::TCombos,
					TEquipment::TSkill,
				>
					.run_if(
						in_state(combo_overview).and(
							added::<ComboOverview<TEquipment::TSkill>>
								.or(changed::<TPlayers::TPlayer, TEquipment::TCombos>),
						),
					),
			)
			.add_systems(
				Update,
				(
					Unusable::visualize_invalid_skill::<
						TPlayers::TPlayer,
						TEquipment::TSkill,
						TEquipment::TSlots,
					>,
					Vertical::dropdown_skill_select_insert::<
						TPlayers::TPlayer,
						TEquipment::TSkill,
						TEquipment::TSlots,
					>,
					Vertical::dropdown_skill_select_click::<
						TPlayers::TPlayer,
						TEquipment::TSkill,
						TEquipment::TCombos,
					>,
					Horizontal::dropdown_skill_select_insert::<
						TPlayers::TPlayer,
						TEquipment::TSkill,
						TEquipment::TSlots,
					>,
					Horizontal::dropdown_skill_select_click::<
						TPlayers::TPlayer,
						TEquipment::TSkill,
						TEquipment::TCombos,
					>,
					select_successor_key::<TPlayers::TPlayer, TEquipment::TCombos>,
					update_combos_view_delete_skill::<
						TPlayers::TPlayer,
						TEquipment::TSkill,
						TEquipment::TCombos,
					>,
				)
					.run_if(in_state(combo_overview)),
			);
	}

	fn inventory_screen(&self, app: &mut App) {
		let inventory = GameState::IngameMenu(MenuState::Inventory);

		app.add_ui::<InventoryScreen, TGraphics::TUiCamera>(inventory)
			.add_systems(
				Update,
				(
					TEquipment::TInventory::set_container_panels::<InventoryPanel, InventoryKey>,
					TEquipment::TSlots::set_container_panels::<InventoryPanel, SlotKey>,
					panel_colors::<InventoryPanel>,
					drag::<TPlayers::TPlayer, InventoryKey>,
					drag::<TPlayers::TPlayer, SlotKey>,
					drop::<TPlayers::TPlayer, InventoryKey, InventoryKey>,
					drop::<TPlayers::TPlayer, SlotKey, SlotKey>,
					drop::<TPlayers::TPlayer, SlotKey, InventoryKey>,
					drop::<TPlayers::TPlayer, InventoryKey, SlotKey>,
				)
					.run_if(in_state(inventory)),
			)
			.add_systems(
				Update,
				(
					swap_equipped_items::<TEquipment::TSlots>.pipe(log_many),
					TEquipment::TInventory::swap_items,
				),
			);
	}

	fn general_systems(&self, app: &mut App) {
		app.add_systems(Update, image_color)
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
