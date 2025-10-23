mod components;
mod events;
mod observers;
mod states;
mod systems;
mod tools;
mod traits;
mod visualization;

#[cfg(debug_assertions)]
mod debug;

use crate::{
	components::{
		DeleteSkill,
		SkillSelectDropdownCommand,
		combo_skill_button::{
			ComboSkillButton,
			DropdownItem,
			DropdownTrigger,
			Horizontal,
			Vertical,
		},
		dispatch_text_color::DispatchTextColor,
		key_select_dropdown_command::KeySelectDropdownCommand,
		label::UILabel,
		ui_disabled::UIDisabled,
	},
	systems::{
		combos::update_combos_view::UpdateComboOverview,
		start_menu_button::set_activity::Activity,
	},
	visualization::unusable::Unusable,
};
use bevy::prelude::*;
use common::{
	resources::Shared,
	states::{
		game_state::{GameState, LoadingEssentialAssets, LoadingGame},
		menu_state::MenuState,
		save_state::SaveState,
	},
	tools::action_key::ActionKey,
	traits::{
		handles_graphics::UiCamera,
		handles_input::{
			HandlesActionKeyButton,
			HandlesInput,
			HandlesInputMut,
			InputMutSystemParam,
			InputSystemParam,
		},
		handles_load_tracking::{
			AssetsProgress,
			DependenciesProgress,
			HandlesLoadTracking,
			LoadGroup,
		},
		handles_loadout::{HandlesLoadout2, LoadoutMutParam, LoadoutReadParam},
		handles_localization::{HandlesLocalization, Token, localized::Localized},
		handles_player::HandlesPlayer,
		handles_saving::HandlesSaving,
		load_asset::Path,
		prefab::AddPrefabObserver,
		register_derived_component::RegisterDerivedComponent,
		thread_safe::ThreadSafe,
	},
};
use components::{
	button_interaction::ButtonInteraction,
	combo_overview::ComboOverview,
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
use states::menus_change_able::MenusChangeable;
use std::{marker::PhantomData, time::Duration};
use systems::{
	adjust_global_z_index::adjust_global_z_index,
	dad::{drag_item::drag_item, drop_item::drop_item},
	image_color::image_color,
	menus_unchangeable_when_present::MenusUnchangeableWhenPresent,
	render_ui::RenderUi,
	set_key_bindings::SetKeyBindings,
	set_state_from_input::set_state_from_input,
	trigger_on_release::TriggerOnRelease,
	update_panels::colors::panel_colors,
};
use traits::{LoadUi, add_dropdown::AddDropdown, add_tooltip::AddTooltip, add_ui::AddUI};

pub struct MenuPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TSavegame, TInput, TLocalization, TGraphics, TPlayers, TLoadout>
	MenuPlugin<(
		TLoading,
		TSavegame,
		TInput,
		TLocalization,
		TGraphics,
		TPlayers,
		TLoadout,
	)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TInput: ThreadSafe + HandlesActionKeyButton + HandlesInput + HandlesInputMut,
	TLocalization: ThreadSafe + HandlesLocalization,
	TGraphics: ThreadSafe + UiCamera,
	TPlayers: ThreadSafe + HandlesPlayer,
	TLoadout: ThreadSafe + HandlesLoadout2,
{
	pub fn from_plugins(
		_: &TLoading,
		_: &TSavegame,
		_: &TInput,
		_: &TLocalization,
		_: &TGraphics,
		_: &TPlayers,
		_: &TLoadout,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TSavegame, TInput, TLocalization, TGraphics, TPlayers, TLoadout>
	MenuPlugin<(
		TLoading,
		TSavegame,
		TInput,
		TLocalization,
		TGraphics,
		TPlayers,
		TLoadout,
	)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TInput: ThreadSafe + HandlesActionKeyButton + HandlesInput + HandlesInputMut,
	TLocalization: ThreadSafe + HandlesLocalization,
	TGraphics: ThreadSafe + UiCamera,
	TPlayers: ThreadSafe + HandlesPlayer,
	TLoadout: ThreadSafe + HandlesLoadout2,
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
				set_state_from_input::<MenuState, InputSystemParam<TInput>>
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
			.add_observer(QuickbarPanel::add_input_control::<TInput::TActionKeyButton>)
			.add_systems(
				Update,
				(
					QuickbarPanel::set_icon::<TPlayers::TPlayer, LoadoutReadParam<TLoadout>>,
					QuickbarPanel::set_color::<
						TPlayers::TPlayer,
						TInput::TActionKeyButton,
						LoadoutReadParam<TLoadout>,
					>,
					panel_colors::<QuickbarPanel>,
				)
					.run_if(in_state(play)),
			);
	}

	fn combo_overview(&self, app: &mut App) {
		type VerticalItem<TId> = ComboSkillButton<DropdownItem<Vertical>, TId>;
		type HorizontalItem<TId> = ComboSkillButton<DropdownItem<Horizontal>, TId>;
		type Trigger<TId> = ComboSkillButton<DropdownTrigger, TId>;

		let combo_overview = GameState::IngameMenu(MenuState::ComboOverview);

		app.add_ui::<ComboOverview<TLoadout::TSkillID>, TLocalization::TLocalizationServer, TGraphics::TUiCamera>(
			combo_overview,
		);
		app.add_dropdown::<TLocalization::TLocalizationServer, KeySelect<AppendSkill>>();
		app.add_dropdown::<TLocalization::TLocalizationServer, VerticalItem<TLoadout::TSkillID>>();
		app.add_dropdown::<TLocalization::TLocalizationServer, HorizontalItem<TLoadout::TSkillID>>(
		);
		app.add_systems(
			Update,
			(
				ComboOverview::update_from::<
					TPlayers::TPlayer,
					LoadoutReadParam<TLoadout>,
					TLoadout::TSkillID,
				>,
				KeySelectDropdownCommand::insert_dropdown::<
					TPlayers::TPlayer,
					LoadoutReadParam<TLoadout>,
				>,
				SkillSelectDropdownCommand::<Vertical>::insert_dropdown::<
					TPlayers::TPlayer,
					LoadoutReadParam<TLoadout>,
					TLoadout::TSkillID,
				>,
				SkillSelectDropdownCommand::<Horizontal>::insert_dropdown::<
					TPlayers::TPlayer,
					LoadoutReadParam<TLoadout>,
					TLoadout::TSkillID,
				>,
				VerticalItem::<TLoadout::TSkillID>::update::<
					TPlayers::TPlayer,
					LoadoutMutParam<TLoadout>,
				>,
				HorizontalItem::<TLoadout::TSkillID>::update::<
					TPlayers::TPlayer,
					LoadoutMutParam<TLoadout>,
				>,
				DeleteSkill::from_combos::<
					TPlayers::TPlayer,
					LoadoutMutParam<TLoadout>,
					TLoadout::TSkillID,
				>,
				Trigger::<TLoadout::TSkillID>::visualize_invalid::<
					Unusable,
					TPlayers::TPlayer,
					LoadoutReadParam<TLoadout>,
				>,
			)
				.chain()
				.run_if(in_state(combo_overview)),
		);
	}

	fn inventory_screen(&self, app: &mut App) {
		let inventory = GameState::IngameMenu(MenuState::Inventory);

		app.add_ui::<InventoryScreen, TLocalization::TLocalizationServer, TGraphics::TUiCamera>(
			inventory,
		)
		.add_systems(
			Update,
			(
				InventoryPanel::set_label::<TPlayers::TPlayer, LoadoutReadParam<TLoadout>>,
				panel_colors::<InventoryPanel>,
				drag_item::<TPlayers::TPlayer>,
				drop_item::<TPlayers::TPlayer, LoadoutMutParam<TLoadout>>,
			)
				.chain()
				.run_if(in_state(inventory)),
		);
	}

	fn settings_screen(&self, app: &mut App) {
		type KeyBindAction = KeyBind<Action<ActionKey>>;
		type KeyBindInput = KeyBind<Input<ActionKey>>;
		type KeyRebindInput = KeyBind<Rebinding<ActionKey>>;

		let settings = GameState::IngameMenu(MenuState::Settings);

		app.register_required_components::<KeyBindInput, Interaction>()
			.register_required_components::<KeyRebindInput, PreventMenuChange>()
			.add_ui::<SettingsScreen, TLocalization::TLocalizationServer, TGraphics::TUiCamera>(
				settings,
			)
			.add_systems(
				Update,
				(
					SettingsScreen::set_key_bindings_from::<InputSystemParam<TInput>>,
					KeyBindAction::render_ui::<TLocalization::TLocalizationServer>,
					KeyBindInput::render_ui::<TLocalization::TLocalizationServer>,
					KeyBindInput::rebind_on_click,
					KeyRebindInput::render_ui::<TLocalization::TLocalizationServer>,
					KeyRebindInput::rebind_apply::<InputMutSystemParam<TInput>>,
				)
					.run_if(in_state(settings)),
			);
	}

	fn general_systems(&self, app: &mut App) {
		let ui_ready = not(in_state(GameState::LoadingEssentialAssets));

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
						InputLabel::icon::<InputSystemParam<TInput>>("icons/keys"),
						Icon::load_image,
						Icon::insert_image,
						UILabel::icon_tooltip,
						UILabel::text,
					)
						.chain(),
				)
					.run_if(ui_ready),
			)
			.add_systems(Last, ButtonInteraction::system);
	}
}

impl<TLoading, TSavegame, TInput, TLocalization, TGraphics, TPlayers, TLoadout> Plugin
	for MenuPlugin<(
		TLoading,
		TSavegame,
		TInput,
		TLocalization,
		TGraphics,
		TPlayers,
		TLoadout,
	)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TInput: ThreadSafe + HandlesActionKeyButton + HandlesInput + HandlesInputMut,
	TLocalization: ThreadSafe + HandlesLocalization,
	TGraphics: ThreadSafe + UiCamera,
	TPlayers: ThreadSafe + HandlesPlayer,
	TLoadout: ThreadSafe + HandlesLoadout2,
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
