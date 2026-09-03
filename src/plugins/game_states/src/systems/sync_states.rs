use crate::{
	resources::game_state_context::GameStateContext,
	states::command_state::CommandState,
	system_params::ui_states::UIStates,
};
use bevy::prelude::*;
use common::prelude::*;
use std::collections::HashSet;

impl GameStateContext {
	pub(crate) fn sync_states(
		mut ctx: ResMut<Self>,
		command_state: Res<State<CommandState>>,
		ui: UIStates,
	) {
		if !command_state.is_changed() && !ui.is_changed() {
			return;
		}

		ctx.command_state = *command_state.get();
		ctx.ui = HashSet::from_iter(IngameUI::iterator().filter(|ui_state| ui.is_on(ui_state)));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		states::{
			command_state::CommandState,
			ui::{ComboOverview, Hud, Inventory, Settings},
		},
		system_params::ui_states::UIStates,
	};
	use bevy::state::{app::StatesPlugin, state::FreelyMutableState};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		UIStates::init(&mut app);
		app.init_state::<CommandState>();
		app.init_resource::<GameStateContext>();
		app.add_systems(Update, GameStateContext::sync_states);

		app
	}

	#[test]
	fn sync_activity() {
		let mut app = setup();
		app.insert_state(CommandState::active(GameStateCommand::Play));

		app.update();

		assert_eq!(
			(
				CommandState::active(GameStateCommand::Play),
				&HashSet::default()
			),
			(
				app.world().resource::<GameStateContext>().command_state,
				&app.world().resource::<GameStateContext>().ui,
			),
		);
	}

	#[test_case(Hud::On, IngameUI::Hud; "hud")]
	#[test_case(Inventory::On, IngameUI::Inventory; "inventory")]
	#[test_case(ComboOverview::On, IngameUI::ComboOverview; "combos")]
	#[test_case(Settings::On, IngameUI::Settings; "settings")]
	fn sync_ui<T>(state: T, ui: IngameUI)
	where
		T: FreelyMutableState,
	{
		let mut app = setup();
		app.insert_state(state);

		app.update();

		assert_eq!(
			&HashSet::from([ui]),
			&app.world().resource::<GameStateContext>().ui
		);
	}

	#[test]
	fn sync_uis() {
		let mut app = setup();
		app.insert_state(Hud::On);
		app.insert_state(ComboOverview::On);

		app.update();

		assert_eq!(
			&HashSet::from([IngameUI::Hud, IngameUI::ComboOverview]),
			&app.world().resource::<GameStateContext>().ui
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		app.insert_state(CommandState::active(GameStateCommand::Play));
		app.insert_state(Inventory::On);

		app.update();
		app.world_mut()
			.resource_mut::<GameStateContext>()
			.command_state = CommandState::active(GameStateCommand::Save);
		app.world_mut().resource_mut::<GameStateContext>().ui = HashSet::from([IngameUI::Hud]);
		app.update();

		assert_eq!(
			(
				CommandState::active(GameStateCommand::Save),
				&HashSet::from([IngameUI::Hud])
			),
			(
				app.world().resource::<GameStateContext>().command_state,
				&app.world().resource::<GameStateContext>().ui
			)
		);
	}

	#[test]
	fn act_again_if_activity_changed() {
		let mut app = setup();
		app.insert_state(CommandState::active(GameStateCommand::Play));

		app.update();
		app.insert_state(CommandState::active(GameStateCommand::Pause));
		app.update();

		assert_eq!(
			CommandState::active(GameStateCommand::Pause),
			app.world().resource::<GameStateContext>().command_state,
		);
	}

	#[test_case(Hud::On, IngameUI::Hud; "hud")]
	#[test_case(Inventory::On, IngameUI::Inventory; "inventory")]
	#[test_case(ComboOverview::On, IngameUI::ComboOverview; "combos")]
	#[test_case(Settings::On, IngameUI::Settings; "settings")]
	fn act_again_if_ui_changed<T>(state: T, ui: IngameUI)
	where
		T: FreelyMutableState,
	{
		let mut app = setup();

		app.update();
		app.insert_state(state);
		app.update();

		assert_eq!(
			&HashSet::from([ui]),
			&app.world().resource::<GameStateContext>().ui,
		);
	}
}
