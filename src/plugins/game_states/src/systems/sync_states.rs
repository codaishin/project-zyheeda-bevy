use crate::{
	resources::game_state_context::GameStatesContext,
	states::activity::Activity,
	system_params::ui_states::UIStates,
};
use bevy::prelude::*;
use common::traits::{
	handles_game_states::{GameState, UIState},
	iteration::IterFinite,
};
use std::{collections::HashSet, iter::once};

impl GameStatesContext {
	pub(crate) fn sync_states(mut ctx: ResMut<Self>, activity: Res<State<Activity>>, ui: UIStates) {
		if !activity.is_changed() && !ui.is_changed() {
			return;
		}

		let states = UIState::iterator()
			.filter(|ui_state| ui.is_on(ui_state))
			.map(GameState::IngameUI)
			.chain(once(GameState::from(activity.get())));

		ctx.states = HashSet::from_iter(states);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		states::{
			activity::Activity,
			ui::{ComboOverview, Hud, Inventory, Settings},
		},
		system_params::ui_states::UIStates,
	};
	use bevy::state::{app::StatesPlugin, state::FreelyMutableState};
	use common::traits::handles_game_states::{ActivityState, GameState, ReadState, UIState};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		UIStates::init(&mut app);
		app.init_state::<Activity>();
		app.init_resource::<GameStatesContext>();
		app.add_systems(Update, GameStatesContext::sync_states);

		app
	}

	#[test_case(Activity::Play, GameState::Activity(ActivityState::Play); "play")]
	#[test_case(Activity::Loading, GameState::Read(ReadState::Loading); "loading")]
	fn sync_activity(state: Activity, game_state: GameState) {
		let mut app = setup();
		app.insert_state(state);

		app.update();

		assert_eq!(
			HashSet::from([game_state]),
			app.world().resource::<GameStatesContext>().states,
		);
	}

	#[test_case(Hud::On, GameState::IngameUI(UIState::Hud); "hud")]
	#[test_case(Inventory::On, GameState::IngameUI(UIState::Inventory); "inventory")]
	#[test_case(ComboOverview::On, GameState::IngameUI(UIState::ComboOverview); "combos")]
	#[test_case(Settings::On, GameState::IngameUI(UIState::Settings); "settings")]
	fn sync_ui<T>(state: T, game_state: GameState)
	where
		T: FreelyMutableState,
	{
		let mut app = setup();
		app.insert_state(state);

		app.update();

		assert_eq!(
			HashSet::from([
				GameState::Activity(ActivityState::LoadingEssentialAssets),
				game_state
			]),
			app.world().resource::<GameStatesContext>().states,
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		app.insert_state(Activity::Play);

		app.update();
		app.world_mut()
			.resource_mut::<GameStatesContext>()
			.states
			.clear();
		app.update();

		assert_eq!(
			HashSet::from([]),
			app.world().resource::<GameStatesContext>().states,
		);
	}

	#[test]
	fn act_again_if_activity_changed() {
		let mut app = setup();
		app.insert_state(Activity::Play);

		app.update();
		app.insert_state(Activity::Paused);
		app.update();

		assert_eq!(
			HashSet::from([GameState::Activity(ActivityState::Paused)]),
			app.world().resource::<GameStatesContext>().states,
		);
	}

	#[test_case(Hud::On, GameState::IngameUI(UIState::Hud); "hud")]
	#[test_case(Inventory::On, GameState::IngameUI(UIState::Inventory); "inventory")]
	#[test_case(ComboOverview::On, GameState::IngameUI(UIState::ComboOverview); "combos")]
	#[test_case(Settings::On, GameState::IngameUI(UIState::Settings); "settings")]
	fn act_again_if_ui_changed<T>(state: T, game_state: GameState)
	where
		T: FreelyMutableState,
	{
		let mut app = setup();
		app.insert_state(Activity::Play);

		app.update();
		app.insert_state(state);
		app.update();

		assert_eq!(
			HashSet::from([GameState::Activity(ActivityState::Play), game_state]),
			app.world().resource::<GameStatesContext>().states,
		);
	}
}
