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
	resources::{Shared, key_map::KeyMap},
	states::{
		game_state::{GameState, LoadingEssentialAssets, LoadingGame},
		menu_state::MenuState,
	},
	systems::insert_required::{InsertOn, InsertRequired},
	tools::{
		change::Change,
		inventory_key::InventoryKey,
		item_description::ItemToken,
		keys::slot::{Combo, SlotKey},
		skill_description::SkillToken,
		skill_execution::SkillExecution,
		skill_icon::SkillIcon,
	},
	traits::{
		handles_combo_menu::{
			ConfigureCombos,
			GetComboAbleSkills,
			GetCombosOrdered,
			HandlesComboMenu,
			NextKeys,
		},
		handles_graphics::{StaticRenderLayers, UiCamera},
		handles_load_tracking::{
			AssetsProgress,
			DependenciesProgress,
			HandlesLoadTracking,
			LoadGroup,
		},
		handles_loadout_menu::{ConfigureInventory, GetItem, HandlesLoadoutMenu, SwapValuesByKey},
		handles_localization::{HandlesLocalization, LocalizeToken, localized::Localized},
		inspect_able::InspectAble,
		load_asset::Path,
		thread_safe::ThreadSafe,
	},
};
use components::{
	button_interaction::ButtonInteraction,
	combo_overview::ComboOverview,
	combo_skill_button::{ComboSkillButton, DropdownItem, Horizontal, Vertical},
	dropdown::Dropdown,
	inventory_panel::InventoryPanel,
	inventory_screen::InventoryScreen,
	key_select::{AppendSkill, KeySelect},
	key_select_dropdown_command::AppendSkillCommand,
	loading_screen::LoadingScreen,
	menu_background::MenuBackground,
	quickbar_panel::QuickbarPanel,
	settings_screen::SettingsScreen,
	start_game::StartGame,
	start_menu::StartMenu,
	start_menu_button::StartMenuButton,
	tooltip::{Tooltip, TooltipUI, TooltipUIControl, TooltipUiConfig},
	ui_overlay::UIOverlay,
};
use events::DropdownEvent;
use resources::equipment_info::EquipmentInfo;
use std::{marker::PhantomData, time::Duration};
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
use traits::{GetLayout, GetRootNode, LoadUi, insert_ui_content::InsertUiContent};
use visualization::unusable::Unusable;

trait AddUI<TState> {
	fn add_ui<TComponent, TLocalizationServer, TGraphics>(&mut self, on_state: TState) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + InsertUiContent,
		TLocalizationServer: Resource + LocalizeToken,
		TGraphics: StaticRenderLayers + 'static;
}

impl<TState> AddUI<TState> for App
where
	TState: States + Copy,
{
	fn add_ui<TComponent, TLocalizationServer, TGraphics>(&mut self, on_state: TState) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + InsertUiContent,
		TLocalizationServer: Resource + LocalizeToken,
		TGraphics: StaticRenderLayers + 'static,
	{
		self.add_systems(
			OnEnter(on_state),
			spawn::<TComponent, AssetServer, TGraphics>,
		)
		.add_systems(OnExit(on_state), despawn::<TComponent>)
		.add_systems(Update, update_children::<TComponent, TLocalizationServer>)
	}
}

trait AddTooltip {
	fn add_tooltip<TLocalization, T>(&mut self) -> &mut Self
	where
		T: TooltipUiConfig + Clone + Sync + Send + 'static,
		Tooltip<T>: InsertUiContent,
		TLocalization: LocalizeToken + Resource;
}

impl AddTooltip for App {
	fn add_tooltip<TLocalization, T>(&mut self) -> &mut Self
	where
		T: TooltipUiConfig + Clone + Sync + Send + 'static,
		Tooltip<T>: InsertUiContent,
		TLocalization: LocalizeToken + Resource,
	{
		self.add_systems(
			Update,
			(
				tooltip::<T, TLocalization, TooltipUI<T>, TooltipUIControl, Window>,
				tooltip_visibility::<Real, T>,
			),
		)
	}
}

trait AddDropdown {
	fn add_dropdown<TLocalization, TItem>(&mut self) -> &mut Self
	where
		TLocalization: LocalizeToken + Resource,
		TItem: InsertUiContent + Sync + Send + 'static,
		Dropdown<TItem>: GetRootNode + GetLayout;
}

impl AddDropdown for App {
	fn add_dropdown<TLocalization, TItem>(&mut self) -> &mut Self
	where
		TLocalization: LocalizeToken + Resource,
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
				.pipe(dropdown_spawn_focused::<TLocalization, TItem>),
		)
	}
}

pub struct MenuPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TLocalization, TGraphics> MenuPlugin<(TLoading, TLocalization, TGraphics)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TLocalization: ThreadSafe + HandlesLocalization,
	TGraphics: ThreadSafe + UiCamera,
{
	pub fn depends_on(_: &TLoading, _: &TLocalization, _: &TGraphics) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TLocalization, TGraphics> MenuPlugin<(TLoading, TLocalization, TGraphics)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TLocalization: ThreadSafe + HandlesLocalization,
	TGraphics: ThreadSafe + UiCamera,
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

		app.add_ui::<StartMenu, TLocalization::TLocalizationServer, TGraphics::TUiCamera>(
			start_menu,
		)
		.add_systems(Update, panel_colors::<StartMenuButton>)
		.add_systems(Update, StartGame::on_release_set(new_game));
	}

	fn loading_screen<TLoadGroup>(&self, app: &mut App)
	where
		TLoadGroup: LoadGroup + ThreadSafe,
	{
		let load_assets = TLoading::processing_state::<TLoadGroup, AssetsProgress>();
		let load_dependencies = TLoading::processing_state::<TLoadGroup, DependenciesProgress>();

		app
			.add_ui::<LoadingScreen<AssetsProgress>, TLocalization::TLocalizationServer, TGraphics::TUiCamera>(
				load_assets
			)
			.add_ui::<LoadingScreen<DependenciesProgress>, TLocalization::TLocalizationServer, TGraphics::TUiCamera>(
				load_dependencies
			);
	}

	fn ui_overlay(&self, app: &mut App) {
		let play = GameState::Play;

		app.add_ui::<UIOverlay, TLocalization::TLocalizationServer, TGraphics::TUiCamera>(play)
			.add_systems(
				Update,
				(
					update_label_text::<KeyMap, TLocalization::TLocalizationServer, QuickbarPanel>,
					panel_colors::<QuickbarPanel>,
				)
					.run_if(in_state(play)),
			)
			.add_systems(
				Update,
				(
					set_ui_mouse_context,
					prime_mouse_context::<KeyMap, QuickbarPanel>,
				),
			);
	}

	fn combo_overview(&self, app: &mut App) {
		app.add_dropdown::<TLocalization::TLocalizationServer, KeySelect<AppendSkill>>();
	}

	fn inventory_screen(&self, app: &mut App) {
		let inventory = GameState::IngameMenu(MenuState::Inventory);

		app.add_ui::<InventoryScreen, TLocalization::TLocalizationServer, TGraphics::TUiCamera>(
			inventory,
		);
	}

	fn settings_screen(&self, app: &mut App) {
		let setup = GameState::IngameMenu(MenuState::Setup);

		app.add_ui::<SettingsScreen, TLocalization::TLocalizationServer, TGraphics::TUiCamera>(
			setup,
		);
	}

	fn general_systems(&self, app: &mut App) {
		app.add_tooltip::<TLocalization::TLocalizationServer, &'static str>()
			.add_tooltip::<TLocalization::TLocalizationServer, String>()
			.add_tooltip::<TLocalization::TLocalizationServer, Localized>()
			.add_systems(
				Update,
				InsertOn::<MenuBackground>::required(MenuBackground::node),
			)
			.add_systems(Update, image_color)
			.add_systems(Update, adjust_global_z_index)
			.add_systems(
				Update,
				insert_key_code_text::<SlotKey, KeyMap, TLocalization::TLocalizationServer>,
			)
			.add_systems(Last, ButtonInteraction::system);
	}
}

impl<TLoading, TLocalization, TGraphics> Plugin for MenuPlugin<(TLoading, TLocalization, TGraphics)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TLocalization: ThreadSafe + HandlesLocalization,
	TGraphics: ThreadSafe + UiCamera,
{
	fn build(&self, app: &mut App) {
		self.resources(app);
		self.events(app);
		self.state_control(app);
		self.start_menu(app);
		self.loading_screen::<LoadingEssentialAssets>(app);
		self.loading_screen::<LoadingGame>(app);
		self.ui_overlay(app);
		self.combo_overview(app);
		self.inventory_screen(app);
		self.settings_screen(app);
		self.general_systems(app);

		#[cfg(debug_assertions)]
		{
			debug::setup_run_time_display::<TLocalization::TLocalizationServer, TGraphics::TUiCamera>(
				app,
			);
			debug::setup_dropdown_test::<TLocalization::TLocalizationServer>(app);
		}
	}
}

impl<TLoading, TLocalization, TGraphics> HandlesLoadoutMenu
	for MenuPlugin<(TLoading, TLocalization, TGraphics)>
where
	TLocalization: HandlesLocalization,
{
	fn loadout_with_swapper<TSwap>() -> impl ConfigureInventory<TSwap>
	where
		TSwap: Component + SwapValuesByKey,
	{
		InventoryConfiguration(PhantomData::<TLocalization::TLocalizationServer>)
	}

	fn configure_quickbar_menu<TContainer, TSystemMarker>(
		app: &mut App,
		get_changed_quickbar: impl IntoSystem<(), Change<TContainer>, TSystemMarker>,
	) where
		TContainer: GetItem<SlotKey> + ThreadSafe,
		TContainer::TItem:
			InspectAble<SkillToken> + InspectAble<SkillIcon> + InspectAble<SkillExecution>,
	{
		let play = GameState::Play;

		app.add_systems(
			Update,
			(
				get_changed_quickbar.pipe(EquipmentInfo::update),
				set_quickbar_icons::<EquipmentInfo<TContainer>>,
				panel_activity_colors_override::<KeyMap, QuickbarPanel, EquipmentInfo<TContainer>>,
			)
				.chain()
				.run_if(in_state(play)),
		);
	}
}

struct InventoryConfiguration<TLocalization>(PhantomData<TLocalization>);

impl<TSwap, TLocalization> ConfigureInventory<TSwap> for InventoryConfiguration<TLocalization>
where
	TSwap: Component + SwapValuesByKey,
	TLocalization: LocalizeToken + Resource,
{
	fn configure<TInventory, TSlots, TSystemMarker1, TSystemMarker2>(
		&self,
		app: &mut App,
		get_changed_inventory: impl IntoSystem<(), Change<TInventory>, TSystemMarker1>,
		get_changed_slots: impl IntoSystem<(), Change<TSlots>, TSystemMarker2>,
	) where
		TInventory: GetItem<InventoryKey> + ThreadSafe,
		TInventory::TItem: InspectAble<ItemToken>,
		TSlots: GetItem<SlotKey> + ThreadSafe,
		TSlots::TItem: InspectAble<ItemToken>,
	{
		let inventory = GameState::IngameMenu(MenuState::Inventory);

		app.add_systems(
			Update,
			(
				get_changed_inventory.pipe(EquipmentInfo::update),
				get_changed_slots.pipe(EquipmentInfo::update),
				InventoryPanel::set_container_panels::<
					TLocalization,
					InventoryKey,
					EquipmentInfo<TInventory>,
				>,
				InventoryPanel::set_container_panels::<TLocalization, SlotKey, EquipmentInfo<TSlots>>,
				panel_colors::<InventoryPanel>,
				drag::<TSwap, InventoryKey>,
				drag::<TSwap, SlotKey>,
				drop::<TSwap, InventoryKey, InventoryKey>,
				drop::<TSwap, InventoryKey, SlotKey>,
				drop::<TSwap, SlotKey, SlotKey>,
				drop::<TSwap, SlotKey, InventoryKey>,
			)
				.chain()
				.run_if(in_state(inventory)),
		);
	}
}

impl<TLoading, TLocalization, TGraphics> HandlesComboMenu
	for MenuPlugin<(TLoading, TLocalization, TGraphics)>
where
	TLocalization: HandlesLocalization + ThreadSafe,
	TGraphics: ThreadSafe + UiCamera,
{
	fn combos_with_skill<TSkill>() -> impl ConfigureCombos<TSkill>
	where
		TSkill: InspectAble<SkillToken> + InspectAble<SkillIcon> + PartialEq + Clone + ThreadSafe,
	{
		ComboConfiguration::<TLocalization, TGraphics>(PhantomData)
	}
}

struct ComboConfiguration<TLocalization, TGraphics>(PhantomData<(TLocalization, TGraphics)>);

impl<TGraphics, TLocalization, TSkill> ConfigureCombos<TSkill>
	for ComboConfiguration<TLocalization, TGraphics>
where
	TGraphics: ThreadSafe + UiCamera,
	TLocalization: HandlesLocalization + ThreadSafe,
	TSkill: InspectAble<SkillToken> + InspectAble<SkillIcon> + Clone + PartialEq + ThreadSafe,
{
	fn configure<TUpdateCombos, TEquipment, M1, M2>(
		&self,
		app: &mut App,
		get_changed_combos: impl IntoSystem<(), Change<TEquipment>, M1>,
		update_combos: TUpdateCombos,
	) where
		TUpdateCombos: IntoSystem<In<Combo<Option<TSkill>>>, (), M2> + Copy,
		TEquipment: GetComboAbleSkills<TSkill> + NextKeys + GetCombosOrdered<TSkill> + ThreadSafe,
	{
		let combo_overview = GameState::IngameMenu(MenuState::ComboOverview);

		app.add_ui::<ComboOverview<TSkill>,TLocalization::TLocalizationServer, TGraphics::TUiCamera>(GameState::IngameMenu(
			MenuState::ComboOverview,
		))
		.add_dropdown::<TLocalization::TLocalizationServer, ComboSkillButton<DropdownItem<Vertical>, TSkill>>()
		.add_dropdown::<TLocalization::TLocalizationServer, ComboSkillButton<DropdownItem<Horizontal>, TSkill>>()
		.add_systems(
			Update,
			(
				get_changed_combos.pipe(EquipmentInfo::update),
				select_successor_key::<EquipmentInfo<TEquipment>>,
				Vertical::dropdown_skill_select_insert::<TSkill, EquipmentInfo<TEquipment>>,
				Horizontal::dropdown_skill_select_insert::<TSkill, EquipmentInfo<TEquipment>>,
				Vertical::dropdown_skill_select_click::<TSkill>.pipe(update_combos),
				Horizontal::dropdown_skill_select_click::<TSkill>.pipe(update_combos),
				update_combos_view_delete_skill::<TSkill>.pipe(update_combos),
				ComboOverview::<TSkill>::update_combos_overview::<TSkill, EquipmentInfo<TEquipment>>,
				Unusable::visualize_invalid_skill::<TSkill, EquipmentInfo<TEquipment>>,
			)
				.chain()
				.run_if(in_state(combo_overview)),
		);
	}
}
