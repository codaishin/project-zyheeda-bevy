mod components;
mod events;
mod observers;
mod states;
mod systems;
mod tools;
mod traits;
mod visualization;

#[cfg(test)]
mod testing;

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
		pause_menu::PauseMenu,
		ui_disabled::UIDisabled,
	},
	systems::{
		combos::update_combos_view::UpdateComboOverview,
		set_activity::set_activity,
		start_menu_button::set_activity::UIActivity,
		toggle_ui::toggle_ui,
	},
	traits::add_ui::AddLoadUI,
	visualization::unusable::Unusable,
};
use bevy::prelude::*;
use common::{prelude::*, tools::path::Path};
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
use events::DropdownMessage;
use macros::asset_path;
use states::menus_change_able::MenusChangeable;
use std::{marker::PhantomData, time::Duration};
use systems::{
	adjust_global_z_index::adjust_global_z_index,
	dad::{drag_item::drag_item, drop_item::drop_item},
	image_color::image_color,
	menus_unchangeable_when_present::MenusUnchangeableWhenPresent,
	render_ui::RenderUi,
	set_key_bindings::SetKeyBindings,
	trigger_on_release::TriggerOnRelease,
	update_panels::colors::panel_colors,
};
use traits::{LoadUi, add_dropdown::AddDropdown, add_tooltip::AddTooltip, add_ui::AddUI};
use zyheeda_core::hash_map;

pub struct UIPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TGameStates, TLoading, TSavegame, TInput, TLocalization, TGraphics, TPlayers, TLoadout>
	UIPlugin<(
		TGameStates,
		TLoading,
		TSavegame,
		TInput,
		TLocalization,
		TGraphics,
		TPlayers,
		TLoadout,
	)>
where
	TGameStates: ThreadSafe + HandlesGameStates + SystemSetDefinition,
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TInput: ThreadSafe + HandlesActionKeyButton + HandlesInput + HandlesInputMut,
	TLocalization: ThreadSafe + HandlesLocalization,
	TGraphics: ThreadSafe + HandlesCameras,
	TPlayers: ThreadSafe + HandlesPlayer,
	TLoadout: ThreadSafe + HandlesLoadout,
{
	#[allow(clippy::too_many_arguments)]
	pub fn from_plugins(
		_: &TGameStates,
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

impl<TGameStates, TLoading, TSavegame, TInput, TLocalization, TGraphics, TPlayers, TLoadout>
	UIPlugin<(
		TGameStates,
		TLoading,
		TSavegame,
		TInput,
		TLocalization,
		TGraphics,
		TPlayers,
		TLoadout,
	)>
where
	TGameStates: ThreadSafe + HandlesGameStates + SystemSetDefinition,
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TInput: ThreadSafe + HandlesActionKeyButton + HandlesInput + HandlesInputMut,
	TLocalization: ThreadSafe + HandlesLocalization,
	TGraphics: ThreadSafe + HandlesCameras,
	TPlayers: ThreadSafe + HandlesPlayer,
	TLoadout: ThreadSafe + HandlesLoadout,
{
	fn resources(&self, app: &mut App) {
		app.init_resource::<Shared<Path, Handle<Image>>>()
			.insert_resource(TooltipUIControl {
				tooltip_delay: Duration::from_millis(500),
			});
	}

	fn messages(&self, app: &mut App) {
		app.add_message::<DropdownMessage>();
	}

	fn state_control(&self, app: &mut App) {
		let changeable = in_state(MenusChangeable(true));
		let in_game = TGameStates::in_game_state([GameStateCommand::Play, GameStateCommand::Pause]);
		let changeable_and_in_game = changeable.and_then(in_game);

		let set_activity = set_activity::<TGameStates::TGameStatesMut, TInput::TInput>(hash_map! {
			ActionKey::Miscellaneous(Miscellaneous::Paused) => GameStateCommand::Pause,
			ActionKey::Save(SaveKey::QuickLoad) => GameStateCommand::Load,
			ActionKey::Save(SaveKey::QuickSave) => GameStateCommand::Save,
		});
		let toggle_ui = toggle_ui::<TGameStates::TGameStatesMut, TInput::TInput>(hash_map! {
			MenuKey::ComboOverview => IngameUI::ComboOverview,
			MenuKey::Inventory => IngameUI::Inventory,
			MenuKey::Settings => IngameUI::Settings,
		});

		app.insert_state(MenusChangeable(true));
		app.add_systems(
			Update,
			(
				PreventMenuChange::menus_unchangeable_when_present,
				(set_activity, toggle_ui).run_if(changeable_and_in_game),
			)
				.chain(),
		);
	}

	fn loading_screen<TLoadGroup>(&self, app: &mut App, load_group: TLoadGroup)
	where
		TLoadGroup: LoadGroup<TLoading::TLoadAssetState>,
	{
		app
			.add_load_ui::<LoadingScreen<AssetsProgress>, TLocalization::TLocalizationServer, TGraphics::TCameraMut, TLoading>(
				load_group
			);
	}

	fn start_menu(&self, app: &mut App) {
		let start_menu = GameStateCommand::StartScreen;
		let load = GameStateCommand::Load;
		let enable_or_disable_quick_load_button = TSavegame::can_quick_load()
			.pipe(|In(can_quick_load)| match can_quick_load {
				true => UIActivity::Enable,
				false => UIActivity::Disable,
			})
			.pipe(StartMenuButton::set_activity(load));

		app.add_prefab_observer::<StartMenuButton, ()>()
			.add_ui::<StartMenu, TLocalization::TLocalizationServer, TGraphics::TCameraMut, TGameStates>(
				start_menu,
			)
			.add_systems(
				Update,
				(
					enable_or_disable_quick_load_button,
					panel_colors::<StartMenuButton>,
					StartMenuButton::trigger_on_release::<TGameStates::TGameStatesMut>,
				)
					.chain()
					.run_if(TGameStates::in_game_state([start_menu]))
					.after_plugin(TGameStates::SYSTEMS),
			);
	}

	fn pause_menu(&self, app: &mut App) {
		app.add_ui::<PauseMenu, TLocalization::TLocalizationServer, TGraphics::TCameraMut, TGameStates>(
			GameStateCommand::Pause,
		);
	}

	fn ui_overlay(&self, app: &mut App) {
		let hud = IngameUI::Hud;

		TGameStates::set_to_not_pause(app, IngameUI::Hud);

		app.add_ui::<UIOverlay, TLocalization::TLocalizationServer, TGraphics::TCameraMut, TGameStates>(hud)
			.add_observer(QuickbarPanel::add_input_control::<TInput::TActionKeyButton>)
			.add_systems(
				Update,
				(
					QuickbarPanel::set_icon::<TPlayers::TPlayer, TLoadout::TLoadout>,
					QuickbarPanel::set_color::<
						TPlayers::TPlayer,
						TInput::TActionKeyButton,
						TLoadout::TLoadout,
					>,
					panel_colors::<QuickbarPanel>,
				)
					.run_if(TGameStates::in_game_state([hud])).after_plugin(TGameStates::SYSTEMS),
			);
	}

	fn combo_overview(&self, app: &mut App) {
		type VerticalItem<TId> = ComboSkillButton<DropdownItem<Vertical>, TId>;
		type HorizontalItem<TId> = ComboSkillButton<DropdownItem<Horizontal>, TId>;
		type Trigger<TId> = ComboSkillButton<DropdownTrigger, TId>;

		let combo_overview = IngameUI::ComboOverview;

		app.add_ui::<ComboOverview<TLoadout::TSkillID>, TLocalization::TLocalizationServer, TGraphics::TCameraMut, TGameStates>(
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
					TLoadout::TLoadout,
					TLoadout::TSkillID,
				>,
				KeySelectDropdownCommand::insert_dropdown::<TPlayers::TPlayer, TLoadout::TLoadout>,
				SkillSelectDropdownCommand::<Vertical>::insert_dropdown::<
					TPlayers::TPlayer,
					TLoadout::TLoadout,
					TLoadout::TSkillID,
				>,
				SkillSelectDropdownCommand::<Horizontal>::insert_dropdown::<
					TPlayers::TPlayer,
					TLoadout::TLoadout,
					TLoadout::TSkillID,
				>,
				VerticalItem::<TLoadout::TSkillID>::update::<
					TPlayers::TPlayer,
					TLoadout::TLoadoutMut,
				>,
				HorizontalItem::<TLoadout::TSkillID>::update::<
					TPlayers::TPlayer,
					TLoadout::TLoadoutMut,
				>,
				DeleteSkill::from_combos::<
					TPlayers::TPlayer,
					TLoadout::TLoadoutMut,
					TLoadout::TSkillID,
				>,
				Trigger::<TLoadout::TSkillID>::visualize_invalid::<
					Unusable,
					TPlayers::TPlayer,
					TLoadout::TLoadout,
				>,
			)
				.chain()
				.run_if(TGameStates::in_game_state([combo_overview]))
				.after_plugin(TGameStates::SYSTEMS),
		);
	}

	fn inventory_screen(&self, app: &mut App) {
		let inventory = IngameUI::Inventory;

		app.add_ui::<InventoryScreen, TLocalization::TLocalizationServer, TGraphics::TCameraMut, TGameStates>(
			inventory,
		)
		.add_systems(
			Update,
			(
				InventoryPanel::set_label::<TPlayers::TPlayer, TLoadout::TLoadout>,
				panel_colors::<InventoryPanel>,
				drag_item::<TPlayers::TPlayer>,
				drop_item::<TPlayers::TPlayer, TLoadout::TLoadoutMut>,
			)
				.chain()
				.run_if(TGameStates::in_game_state([inventory])).after_plugin(TGameStates::SYSTEMS),
		);
	}

	fn settings_screen(&self, app: &mut App) {
		type KeyBindAction = KeyBind<Action<ActionKey>>;
		type KeyBindInput = KeyBind<Input<ActionKey>>;
		type KeyRebindInput = KeyBind<Rebinding<ActionKey>>;

		let settings = IngameUI::Settings;

		app.register_required_components::<KeyBindInput, Interaction>()
			.register_required_components::<KeyRebindInput, PreventMenuChange>()
			.add_ui::<SettingsScreen, TLocalization::TLocalizationServer, TGraphics::TCameraMut, TGameStates>(
				settings,
			)
			.add_systems(
				Update,
				(
					SettingsScreen::set_key_bindings_from::<TInput::TInput>,
					KeyBindAction::render_ui::<TLocalization::TLocalizationServer>,
					KeyBindInput::render_ui::<TLocalization::TLocalizationServer>,
					KeyBindInput::rebind_on_click,
					KeyRebindInput::render_ui::<TLocalization::TLocalizationServer>,
					KeyRebindInput::rebind_apply::<TInput::TInputMut>,
				)
					.run_if(TGameStates::in_game_state([settings])).after_plugin(TGameStates::SYSTEMS),
			);
	}

	fn general_systems(&self, app: &mut App) {
		let ui_ready = not(TLoading::is_loading(LoadingEssentialAssets));

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
						InputLabel::icon::<TInput::TInput>(asset_path!("generic/icons")),
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

impl<TGameStates, TLoading, TSavegame, TInput, TLocalization, TGraphics, TPlayers, TLoadout> Plugin
	for UIPlugin<(
		TGameStates,
		TLoading,
		TSavegame,
		TInput,
		TLocalization,
		TGraphics,
		TPlayers,
		TLoadout,
	)>
where
	TGameStates: ThreadSafe + HandlesGameStates + SystemSetDefinition,
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TInput: ThreadSafe + HandlesActionKeyButton + HandlesInput + HandlesInputMut,
	TLocalization: ThreadSafe + HandlesLocalization,
	TGraphics: ThreadSafe + HandlesCameras,
	TPlayers: ThreadSafe + HandlesPlayer,
	TLoadout: ThreadSafe + HandlesLoadout,
{
	fn build(&self, app: &mut App) {
		self.resources(app);
		self.messages(app);
		self.state_control(app);
		self.loading_screen(app, LoadingEssentialAssets);
		self.loading_screen(app, LoadingGame);
		self.start_menu(app);
		self.pause_menu(app);
		self.ui_overlay(app);
		self.combo_overview(app);
		self.inventory_screen(app);
		self.settings_screen(app);
		self.general_systems(app);

		#[cfg(debug_assertions)]
		debug::setup_dropdown_test::<TLocalization::TLocalizationServer>(app);
	}
}
