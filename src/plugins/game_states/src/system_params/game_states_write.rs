use crate::{
	resources::game_state_context::GameStatesContext,
	states::activity::Activity,
	system_params::ui_states::UIStates,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::handles_game_states::{
	ActivityState,
	AddGameState,
	GameState,
	GameStates,
	RemoveGameState,
	UIState,
};
use std::collections::HashSet;

#[derive(SystemParam)]
pub struct GameStatesWrite<'w> {
	ctx: ResMut<'w, GameStatesContext>,
	activity: ResMut<'w, NextState<Activity>>,
	ui: UIStates<'w>,
}

impl GameStates for GameStatesWrite<'_> {
	fn game_states(&self) -> &HashSet<GameState> {
		&self.ctx.states
	}
}

impl AddGameState<ActivityState> for GameStatesWrite<'_> {
	fn add_game_state(&mut self, activity: ActivityState) {
		self.activity.set(Activity::from(activity));
	}
}

impl AddGameState<UIState> for GameStatesWrite<'_> {
	fn add_game_state(&mut self, ui: UIState) {
		self.ui.set_on(ui);
	}
}

impl RemoveGameState<UIState> for GameStatesWrite<'_> {
	fn remove_game_state(&mut self, ui: &UIState) {
		self.ui.set_off(ui);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::states::ui::{ComboOverview, Hud, Inventory, Settings};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		state::{app::StatesPlugin, state::FreelyMutableState},
	};
	use common::traits::{
		handles_game_states::{ActivityState, UIState},
		thread_safe::ThreadSafe,
	};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		app.init_state::<Activity>();
		UIStates::init(&mut app);
		app.init_resource::<GameStatesContext>();

		app
	}

	macro_rules! assert_state_eq {
		($left:expr, $right:expr) => {{
			use NextState::*;

			match ($left, $right) {
				(Unchanged, Unchanged) => {}
				(Pending(left), Pending(right)) => assert_eq!(left, right),
				(PendingIfNeq(left), PendingIfNeq(right)) => {
					assert_eq!(left, right)
				}
				(left, right) => {
					panic!("assertion: `left == right` failed\n  left: {left:?}\n right: {right:?}")
				}
			}
		}};
	}

	#[test_case(ActivityState::Paused, Activity::Paused; "paused")]
	#[test_case(UIState::Hud, Hud::On; "hud")]
	#[test_case(UIState::Inventory, Inventory::On; "inventory")]
	#[test_case(UIState::ComboOverview, ComboOverview::On; "combos")]
	#[test_case(UIState::Settings, Settings::On; "settings")]
	fn add_state<T, U>(state: T, expected: U) -> Result<(), RunSystemError>
	where
		for<'w> GameStatesWrite<'w>: AddGameState<T>,
		T: Copy + ThreadSafe,
		U: FreelyMutableState,
	{
		let mut app = setup();

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWrite| {
				w.add_game_state(state);
			})?;

		assert_state_eq!(
			&NextState::Pending(expected),
			app.world().resource::<NextState<U>>()
		);
		Ok(())
	}

	#[test_case(UIState::Hud, Hud::Off; "hud")]
	#[test_case(UIState::Inventory, Inventory::Off; "inventory")]
	#[test_case(UIState::ComboOverview, ComboOverview::Off; "combos")]
	#[test_case(UIState::Settings, Settings::Off; "settings")]
	fn remove_state<U>(state: UIState, expected: U) -> Result<(), RunSystemError>
	where
		U: FreelyMutableState,
	{
		let mut app = setup();

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWrite| {
				w.remove_game_state(&state);
			})?;

		assert_state_eq!(
			&NextState::Pending(expected),
			app.world().resource::<NextState<U>>()
		);
		Ok(())
	}
}
