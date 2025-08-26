mod components;
mod events;
mod observers;
mod resources;
mod states;
mod systems;
mod tools;
mod traits;
mod visualization;

#[cfg(debug_assertions)]
mod debug;

use crate::{
	components::{dispatch_text_color::DispatchTextColor, label::UILabel, ui_disabled::UIDisabled},
	systems::{
		combos::visualize_invalid_skill::VisualizeInvalidSkill,
		dropdown::dropdown_skill_select_insert::DropdownSkillSelectInsert,
		start_menu_button::set_activity::Activity,
		update_panels::set_container_panels::SetContainerPanels,
	},
};
use bevy::{ecs::component::Mutable, prelude::*};
use common::{
	components::ui_input_primer::UiInputPrimer,
	resources::Shared,
	states::{
		game_state::{GameState, LoadingEssentialAssets, LoadingGame},
		menu_state::MenuState,
		save_state::SaveState,
	},
	tools::{
		action_key::{ActionKey, slot::PlayerSlot, user_input::UserInput},
		change::Change,
		inventory_key::InventoryKey,
		skill_execution::SkillExecution,
	},
	traits::{
		accessors::get::RefInto,
		handles_combo_menu::{
			Combo,
			ConfigurePlayerCombos,
			GetComboAblePlayerSkills,
			GetCombosOrdered,
			HandlesComboMenu,
			NextKeys,
		},
		handles_graphics::UiCamera,
		handles_load_tracking::{
			AssetsProgress,
			DependenciesProgress,
			HandlesLoadTracking,
			LoadGroup,
		},
		handles_loadout_menu::{ConfigureInventory, GetItem, HandlesLoadoutMenu, SwapValuesByKey},
		handles_localization::{HandlesLocalization, Localize, Token, localized::Localized},
		handles_saving::HandlesSaving,
		handles_settings::HandlesSettings,
		load_asset::Path,
		prefab::AddPrefabObserver,
		register_derived_component::RegisterDerivedComponent,
		thread_safe::ThreadSafe,
	},
};
use components::{
	button_interaction::ButtonInteraction,
	combo_overview::ComboOverview,
	combo_skill_button::{ComboSkillButton, DropdownItem, Horizontal, Vertical},
	dropdown::Dropdown,
	icon::Icon,
	input_label::InputLabel,
	inventory_panel::InventoryPanel,
	inventory_screen::InventoryScreen,
	key_select::{AppendSkill, KeySelect},
	key_select_dropdown_command::AppendSkillCommand,
	loading_screen::LoadingScreen,
	menu_background::MenuBackground,
	prevent_menu_change::PreventMenuChange,
	quickbar_panel::QuickbarPanel,
	settings_screen::{
		SettingsScreen,
		key_bind::{KeyBind, action::Action, input::Input, rebinding::Rebinding},
	},
	start_menu::StartMenu,
	start_menu_button::StartMenuButton,
	tooltip::{Tooltip, TooltipUIControl},
	ui_overlay::UIOverlay,
};
use events::DropdownEvent;
use resources::equipment_info::EquipmentInfo;
use states::menus_change_able::MenusChangeable;
use std::{marker::PhantomData, time::Duration};
use systems::{
	adjust_global_z_index::adjust_global_z_index,
	combos::{
		dropdown_skill_select_click::DropdownSkillSelectClick,
		update_combos_view::UpdateComboOverview,
		update_combos_view_delete_skill::update_combos_view_delete_skill,
	},
	dad::{drag::drag, drop::drop},
	dropdown::select_successor_key::select_successor_key,
	image_color::image_color,
	menus_unchangeable_when_present::MenusUnchangeableWhenPresent,
	render_ui::RenderUi,
	set_key_bindings::SetKeyBindings,
	set_state_from_input::set_state_from_input,
	trigger_on_release::TriggerOnRelease,
	update_panels::{
		activity_colors_override::panel_activity_colors_override,
		colors::panel_colors,
		set_quickbar_icons::set_quickbar_icons,
	},
};
use traits::{LoadUi, add_dropdown::AddDropdown, add_tooltip::AddTooltip, add_ui::AddUI};
use visualization::unusable::Unusable;

pub struct MenuPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TSavegame, TSettings, TLocalization, TGraphics>
	MenuPlugin<(TLoading, TSavegame, TSettings, TLocalization, TGraphics)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TSettings: ThreadSafe + HandlesSettings,
	TLocalization: ThreadSafe + HandlesLocalization,
	TGraphics: ThreadSafe + UiCamera,
{
	pub fn from_plugins(
		_: &TLoading,
		_: &TSavegame,
		_: &TSettings,
		_: &TLocalization,
		_: &TGraphics,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TSavegame, TSettings, TLocalization, TGraphics>
	MenuPlugin<(TLoading, TSavegame, TSettings, TLocalization, TGraphics)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TSettings: ThreadSafe + HandlesSettings,
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
		let changeable = in_state(MenusChangeable(true));
		let loading_essentials = in_state(GameState::LoadingEssentialAssets);
		let changeable_and_not_loading = changeable.and(not(loading_essentials));

		app.insert_state(MenusChangeable(true));
		app.add_systems(
			Update,
			(
				PreventMenuChange::menus_unchangeable_when_present,
				set_state_from_input::<GameState, MenuState, TSettings::TKeyMap<MenuState>>
					.run_if(changeable_and_not_loading),
			)
				.chain(),
		);
	}

	fn start_menu(&self, app: &mut App) {
		let start_menu = GameState::StartMenu;
		let quick_load = GameState::Save(SaveState::AttemptLoad);
		let enable_or_disable_quick_load_button = TSavegame::can_quick_load()
			.pipe(|In(can_quick_load)| match can_quick_load {
				true => Activity::Enable,
				false => Activity::Disable,
			})
			.pipe(StartMenuButton::set_activity(quick_load));

		app.add_prefab_observer::<StartMenuButton, ()>()
			.add_ui::<StartMenu, TLocalization::TLocalizationServer, TGraphics::TUiCamera>(
				start_menu,
			)
			.add_systems(
				Update,
				(
					enable_or_disable_quick_load_button,
					panel_colors::<StartMenuButton>,
					StartMenuButton::trigger_on_release,
				)
					.chain()
					.run_if(in_state(start_menu)),
			);
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
					QuickbarPanel::add_quickbar_primer::<TSettings::TKeyMap<PlayerSlot>>,
					panel_colors::<QuickbarPanel>,
				)
					.run_if(in_state(play)),
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
		type KeyBindAction = KeyBind<Action<ActionKey>>;
		type KeyBindInput = KeyBind<Input<ActionKey, UserInput>>;
		type KeyRebindInput = KeyBind<Rebinding<ActionKey, UserInput>>;

		let settings = GameState::IngameMenu(MenuState::Settings);

		app.register_required_components::<KeyBindInput, Interaction>()
			.register_required_components::<KeyRebindInput, PreventMenuChange>()
			.add_ui::<SettingsScreen, TLocalization::TLocalizationServer, TGraphics::TUiCamera>(
				settings,
			)
			.add_systems(
				Update,
				(
					SettingsScreen::set_key_bindings_from::<TSettings::TKeyMap<ActionKey>>,
					KeyBindAction::render_ui::<TLocalization::TLocalizationServer>,
					KeyBindInput::render_ui::<TLocalization::TLocalizationServer>,
					KeyBindInput::rebind_on_click,
					KeyRebindInput::render_ui::<TLocalization::TLocalizationServer>,
					KeyRebindInput::rebind_apply::<TSettings::TKeyMap<ActionKey>>,
				)
					.run_if(in_state(settings)),
			);
	}

	fn general_systems(&self, app: &mut App) {
		let ui_ready = not(in_state(GameState::LoadingEssentialAssets));
		let input_label_icons = InputLabel::<PlayerSlot>::icon::<TSettings::TKeyMap<PlayerSlot>>;

		app.register_derived_component::<MenuBackground, Node>()
			.add_observer(UILabel::localize::<TLocalization::TLocalizationServer>)
			.add_tooltip::<TLocalization::TLocalizationServer, Token>()
			.add_tooltip::<TLocalization::TLocalizationServer, Localized>()
			.add_systems(
				Update,
				(
					image_color,
					adjust_global_z_index,
					DispatchTextColor::apply,
					UIDisabled::apply,
					(
						input_label_icons("icons/keys"),
						Icon::load_image,
						Icon::insert_image,
						Icon::insert_image_tooltip,
						Icon::insert_fallback_text,
					)
						.chain(),
				)
					.run_if(ui_ready),
			)
			.add_systems(Last, ButtonInteraction::system);
	}
}

impl<TLoading, TSavegame, TSettings, TLocalization, TGraphics> Plugin
	for MenuPlugin<(TLoading, TSavegame, TSettings, TLocalization, TGraphics)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TSettings: ThreadSafe + HandlesSettings,
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

impl<TLoading, TSavegame, TSettings, TLocalization, TGraphics> HandlesLoadoutMenu
	for MenuPlugin<(TLoading, TSavegame, TSettings, TLocalization, TGraphics)>
where
	TSettings: HandlesSettings,
	TLocalization: HandlesLocalization,
{
	fn loadout_with_swapper<TSwap>() -> impl ConfigureInventory<TSwap>
	where
		TSwap: Component<Mutability = Mutable> + SwapValuesByKey,
	{
		InventoryConfiguration(PhantomData::<TLocalization::TLocalizationServer>)
	}

	fn configure_quickbar_menu<TQuickbar, TSystemMarker>(
		app: &mut App,
		get_changed_quickbar: impl IntoSystem<(), Change<TQuickbar>, TSystemMarker>,
	) where
		TQuickbar: GetItem<PlayerSlot> + ThreadSafe,
		TQuickbar::TItem: for<'a> RefInto<'a, &'a Token>
			+ for<'a> RefInto<'a, &'a Option<Handle<Image>>>
			+ for<'a> RefInto<'a, &'a SkillExecution>,
	{
		let play = GameState::Play;

		app.add_systems(
			Update,
			(
				get_changed_quickbar.pipe(EquipmentInfo::update),
				set_quickbar_icons::<EquipmentInfo<TQuickbar>>,
				panel_activity_colors_override::<
					TSettings::TKeyMap<PlayerSlot>,
					QuickbarPanel,
					UiInputPrimer,
					EquipmentInfo<TQuickbar>,
				>,
			)
				.chain()
				.run_if(in_state(play)),
		);
	}
}

struct InventoryConfiguration<TLocalization>(PhantomData<TLocalization>);

impl<TSwap, TLocalization> ConfigureInventory<TSwap> for InventoryConfiguration<TLocalization>
where
	TSwap: Component<Mutability = Mutable> + SwapValuesByKey,
	TLocalization: Localize + Resource,
{
	fn configure<TInventory, TSlots, TSystemMarker1, TSystemMarker2>(
		&self,
		app: &mut App,
		get_changed_inventory: impl IntoSystem<(), Change<TInventory>, TSystemMarker1>,
		get_changed_slots: impl IntoSystem<(), Change<TSlots>, TSystemMarker2>,
	) where
		TInventory: GetItem<InventoryKey> + ThreadSafe,
		TInventory::TItem: for<'a> RefInto<'a, &'a Token>,
		TSlots: GetItem<PlayerSlot> + ThreadSafe,
		TSlots::TItem: for<'a> RefInto<'a, &'a Token>,
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
				InventoryPanel::set_container_panels::<
					TLocalization,
					PlayerSlot,
					EquipmentInfo<TSlots>,
				>,
				panel_colors::<InventoryPanel>,
				drag::<TSwap, InventoryKey>,
				drag::<TSwap, PlayerSlot>,
				drop::<TSwap, InventoryKey, InventoryKey>,
				drop::<TSwap, InventoryKey, PlayerSlot>,
				drop::<TSwap, PlayerSlot, PlayerSlot>,
				drop::<TSwap, PlayerSlot, InventoryKey>,
			)
				.chain()
				.run_if(in_state(inventory)),
		);
	}
}

impl<TLoading, TSavegame, TSettings, TLocalization, TGraphics> HandlesComboMenu
	for MenuPlugin<(TLoading, TSavegame, TSettings, TLocalization, TGraphics)>
where
	TLocalization: HandlesLocalization + ThreadSafe,
	TGraphics: ThreadSafe + UiCamera,
{
	fn combos_with_skill<TSkill>() -> impl ConfigurePlayerCombos<TSkill>
	where
		TSkill: PartialEq
			+ Clone
			+ ThreadSafe
			+ for<'a> RefInto<'a, &'a Token>
			+ for<'a> RefInto<'a, &'a Option<Handle<Image>>>,
	{
		ComboConfiguration::<TLocalization, TGraphics>(PhantomData)
	}
}

struct ComboConfiguration<TLocalization, TGraphics>(PhantomData<(TLocalization, TGraphics)>);

impl<TGraphics, TLocalization, TSkill> ConfigurePlayerCombos<TSkill>
	for ComboConfiguration<TLocalization, TGraphics>
where
	TGraphics: ThreadSafe + UiCamera,
	TLocalization: HandlesLocalization + ThreadSafe,
	TSkill: PartialEq
		+ Clone
		+ ThreadSafe
		+ for<'a> RefInto<'a, &'a Token>
		+ for<'a> RefInto<'a, &'a Option<Handle<Image>>>,
{
	fn configure<TUpdateCombos, TCombos, M1, M2>(
		&self,
		app: &mut App,
		get_changed_combos: impl IntoSystem<(), Change<TCombos>, M1>,
		update_combos: TUpdateCombos,
	) where
		TUpdateCombos: IntoSystem<In<Combo<PlayerSlot, Option<TSkill>>>, (), M2> + Copy,
		TCombos: GetCombosOrdered<TSkill, PlayerSlot>
			+ GetComboAblePlayerSkills<TSkill>
			+ NextKeys<PlayerSlot>
			+ ThreadSafe,
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
				select_successor_key::<EquipmentInfo<TCombos>>,
				Vertical::dropdown_skill_select_insert::<TSkill, EquipmentInfo<TCombos>>,
				Horizontal::dropdown_skill_select_insert::<TSkill, EquipmentInfo<TCombos>>,
				Vertical::dropdown_skill_select_click::<TSkill>.pipe(update_combos),
				Horizontal::dropdown_skill_select_click::<TSkill>.pipe(update_combos),
				update_combos_view_delete_skill::<TSkill>.pipe(update_combos),
				ComboOverview::<TSkill>::update_combos_overview::<TSkill, EquipmentInfo<TCombos>>,
				Unusable::visualize_invalid_skill::<TSkill, EquipmentInfo<TCombos>>,
			)
				.chain()
				.run_if(in_state(combo_overview)),
		);
	}
}
