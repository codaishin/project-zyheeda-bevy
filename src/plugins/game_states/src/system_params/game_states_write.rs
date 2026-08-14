use crate::{
	resources::game_state_context::GameStateContext,
	states::activity::Activity,
	system_params::ui_states::UIStatesMut,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::prelude::*;
use std::collections::HashSet;

#[derive(SystemParam)]
pub struct GameStatesWrite<'w, 's> {
	current: Res<'w, GameStateContext>,
	activity: ResMut<'w, NextState<Activity>>,
	ui: UIStatesMut<'w>,
	next: Local<'s, Option<NextGameStates>>,
}

impl GameStates for GameStatesWrite<'_, '_> {
	fn game_states(&self) -> GameStateCollection<'_> {
		GameStateCollection {
			activity: self.current.activity,
			ui: &self.current.ui,
		}
	}
}

impl GameStatesMut for GameStatesWrite<'_, '_> {
	fn game_states_mut(&mut self) -> GameStateCollectionMut<'_> {
		GameStateCollectionMut {
			current: GameStateCollection {
				activity: self.current.activity,
				ui: &self.current.ui,
			},
			next: self.next.get_or_insert(NextGameStates {
				activity: self.current.activity,
				ui: self.current.ui.clone(),
			}),
		}
	}
}

impl Drop for GameStatesWrite<'_, '_> {
	fn drop(&mut self) {
		let Some(next) = self.next.take() else {
			return;
		};

		if next.activity != self.current.activity {
			self.activity.set(Activity(next.activity));
		}

		for state in UIState::iterator() {
			match Change::of(&state, &self.current.ui, &next.ui) {
				Some(Change::Added) => self.ui.set_on(state),
				Some(Change::Removed) => self.ui.set_off(&state),
				None => continue,
			}
		}
	}
}

enum Change {
	Added,
	Removed,
}

impl Change {
	fn of(
		state: &UIState,
		current: &HashSet<UIState>,
		queued: &HashSet<UIState>,
	) -> Option<Change> {
		match (current.contains(state), queued.contains(state)) {
			(true, false) => Some(Change::Removed),
			(false, true) => Some(Change::Added),
			_ => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		states::ui::{ComboOverview, Hud, Inventory, Settings},
		system_params::ui_states::UIStates,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		state::{app::StatesPlugin, state::FreelyMutableState},
	};
	use std::marker::PhantomData;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		app.init_state::<Activity>();
		UIStates::init(&mut app);
		app.init_resource::<GameStateContext>();

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

	#[test]
	fn next_matches_current() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().resource_mut::<GameStateContext>().activity = ActivityState::NewGame;
		app.world_mut().resource_mut::<GameStateContext>().ui =
			HashSet::from([UIState::Hud, UIState::Settings]);

		let queued = app
			.world_mut()
			.run_system_once(move |mut w: GameStatesWrite| w.game_states_mut().next.clone())?;

		assert_eq!(
			NextGameStates {
				activity: ActivityState::NewGame,
				ui: HashSet::from([UIState::Hud, UIState::Settings])
			},
			queued
		);
		Ok(())
	}

	#[test]
	fn next_matches_changed_current_state() -> Result<(), RunSystemError> {
		#[derive(Resource, Debug, PartialEq)]
		struct _Result(NextGameStates);

		fn get_queued(mut w: GameStatesWrite, mut c: Commands) {
			c.insert_resource(_Result(w.game_states_mut().next.clone()));
		}

		let mut app = setup();
		app.add_systems(Update, get_queued);

		app.update();
		app.world_mut().resource_mut::<GameStateContext>().activity = ActivityState::NewGame;
		app.world_mut().resource_mut::<GameStateContext>().ui =
			HashSet::from([UIState::Hud, UIState::Settings]);
		app.update();

		assert_eq!(
			&_Result(NextGameStates {
				activity: ActivityState::NewGame,
				ui: HashSet::from([UIState::Hud, UIState::Settings])
			}),
			app.world().resource::<_Result>()
		);
		Ok(())
	}

	#[test]
	fn retrieving_next_multiple_times_retains_changes() -> Result<(), RunSystemError> {
		let mut app = setup();

		let queued = app
			.world_mut()
			.run_system_once(move |mut w: GameStatesWrite| {
				w.game_states_mut().next.activity = ActivityState::NewGame;
				w.game_states_mut().next.ui = HashSet::from([UIState::Hud, UIState::Settings]);
				w.game_states_mut().next.clone()
			})?;

		assert_eq!(
			NextGameStates {
				activity: ActivityState::NewGame,
				ui: HashSet::from([UIState::Hud, UIState::Settings])
			},
			queued
		);
		Ok(())
	}

	#[test]
	fn set_activity() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWrite| {
				w.game_states_mut().next.activity = ActivityState::Paused;
			})?;

		assert_state_eq!(
			&NextState::Pending(Activity(ActivityState::Paused)),
			app.world().resource::<NextState<Activity>>()
		);
		Ok(())
	}

	#[test_case(UIState::Hud, Hud::On; "hud")]
	#[test_case(UIState::Inventory, Inventory::On; "inventory")]
	#[test_case(UIState::ComboOverview, ComboOverview::On; "combos")]
	#[test_case(UIState::Settings, Settings::On; "settings")]
	fn add_ui<TState>(state: UIState, expected: TState) -> Result<(), RunSystemError>
	where
		TState: FreelyMutableState,
	{
		let mut app = setup();

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWrite| {
				w.game_states_mut().next.ui.insert(state);
			})?;

		assert_state_eq!(
			&NextState::Pending(expected),
			app.world().resource::<NextState<TState>>()
		);
		Ok(())
	}

	#[test_case(UIState::Hud, Hud::Off; "hud")]
	#[test_case(UIState::Inventory, Inventory::Off; "inventory")]
	#[test_case(UIState::ComboOverview, ComboOverview::Off; "combos")]
	#[test_case(UIState::Settings, Settings::Off; "settings")]
	fn remove_ui<TState>(state: UIState, expected: TState) -> Result<(), RunSystemError>
	where
		TState: FreelyMutableState,
	{
		let mut app = setup();
		app.world_mut()
			.resource_mut::<GameStateContext>()
			.ui
			.insert(state);

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWrite| {
				w.game_states_mut().next.ui.remove(&state);
			})?;

		assert_state_eq!(
			&NextState::Pending(expected),
			app.world().resource::<NextState<TState>>()
		);
		Ok(())
	}

	#[test_case(PhantomData::<Activity>; "activity")]
	#[test_case(PhantomData::<Hud>; "hud")]
	#[test_case(PhantomData::<Inventory>; "inventory")]
	#[test_case(PhantomData::<ComboOverview>; "combos")]
	#[test_case(PhantomData::<Settings>; "settings")]
	fn do_nothing_if_not_changed<TState>(_: PhantomData<TState>) -> Result<(), RunSystemError>
	where
		TState: FreelyMutableState,
	{
		let mut app = setup();

		app.world_mut().run_system_once(|mut p: GameStatesWrite| {
			p.game_states_mut();
		})?;

		assert_state_eq!(
			&NextState::<TState>::Unchanged,
			app.world().resource::<NextState<TState>>()
		);
		Ok(())
	}
}
