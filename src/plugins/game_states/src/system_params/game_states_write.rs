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
	activity_change: Local<'s, Option<SettableState>>,
	ui_change: Local<'s, Option<HashSet<UIState>>>,
}

impl GameStatesWrite<'_, '_> {
	fn drain_activity_change(&mut self) {
		let Some(activity) = self.activity_change.take() else {
			return;
		};

		if activity == self.current.activity {
			return;
		}

		self.activity
			.set(Activity(ActivityState::Settable(activity)));
	}

	fn drain_ui_change(&mut self) {
		let Some(ui) = self.ui_change.take() else {
			return;
		};

		let detect_change = |state| Change::of(state, &self.current.ui, &ui);
		let changes = UIState::iterator().filter_map(detect_change);

		for change in changes {
			match change {
				Change::Added(state) => self.ui.set_on(state),
				Change::Removed(state) => self.ui.set_off(&state),
			}
		}
	}
}

impl GameStates for GameStatesWrite<'_, '_> {
	fn activity(&self) -> ActivityState {
		self.current.activity
	}

	fn ui(&self) -> &'_ HashSet<UIState> {
		&self.current.ui
	}
}

impl GameStatesMut for GameStatesWrite<'_, '_> {
	fn set_activity(&mut self, activity: SettableState) {
		*self.activity_change = Some(activity);
	}

	fn ui_mut(&mut self) -> &'_ mut HashSet<UIState> {
		self.ui_change.get_or_insert(self.current.ui.clone())
	}
}

impl Drop for GameStatesWrite<'_, '_> {
	fn drop(&mut self) {
		self.drain_activity_change();
		self.drain_ui_change();
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct NextGameStates {
	activity: SettableState,
	ui: HashSet<UIState>,
}

impl GameStatesMut for &mut NextGameStates {
	fn set_activity(&mut self, activity: SettableState) {
		self.activity = activity;
	}

	fn ui_mut(&mut self) -> &'_ mut HashSet<UIState> {
		&mut self.ui
	}
}

enum Change {
	Added(UIState),
	Removed(UIState),
}

impl Change {
	fn of(state: UIState, current: &HashSet<UIState>, queued: &HashSet<UIState>) -> Option<Change> {
		match (current.contains(&state), queued.contains(&state)) {
			(true, false) => Some(Change::Removed(state)),
			(false, true) => Some(Change::Added(state)),
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
	fn set_activity() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWrite| {
				w.set_activity(SettableState::Paused);
			})?;

		assert_state_eq!(
			&NextState::Pending(Activity(ActivityState::Settable(SettableState::Paused))),
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
				w.ui_mut().insert(state);
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
				w.ui_mut().remove(&state);
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
		app.world_mut().resource_mut::<GameStateContext>().activity =
			ActivityState::Settable(SettableState::Play);
		app.world_mut().resource_mut::<GameStateContext>().ui =
			HashSet::from([UIState::Hud, UIState::Settings]);

		app.world_mut().run_system_once(|mut p: GameStatesWrite| {
			p.set_activity(SettableState::Play);
			*p.ui_mut() = HashSet::from([UIState::Hud, UIState::Settings]);
		})?;

		assert_state_eq!(
			&NextState::<TState>::Unchanged,
			app.world().resource::<NextState<TState>>()
		);
		Ok(())
	}

	fn execute_once(
		exec: impl Fn(&mut GameStatesWrite),
	) -> impl FnMut(GameStatesWrite, Local<bool>) {
		move |mut p: GameStatesWrite, mut done: Local<bool>| {
			if *done {
				return;
			}

			exec(&mut p);
			*done = true;
		}
	}

	#[test]
	fn do_not_repeat_stale_activity_change() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.add_systems(
			Update,
			execute_once(|p| p.set_activity(SettableState::Play)),
		);
		app.update();
		app.update();

		assert_state_eq!(
			&NextState::<Activity>::Unchanged,
			app.world().resource::<NextState<Activity>>()
		);
		Ok(())
	}

	#[test_case(UIState::Hud, PhantomData::<Hud>; "hud")]
	#[test_case(UIState::Inventory, PhantomData::<Inventory>; "inventory")]
	#[test_case(UIState::ComboOverview, PhantomData::<ComboOverview>; "combos")]
	#[test_case(UIState::Settings, PhantomData::<Settings>; "settings")]
	fn do_not_repeat_stale_ui_change<T>(
		ui_state: UIState,
		_: PhantomData<T>,
	) -> Result<(), RunSystemError>
	where
		T: FreelyMutableState,
	{
		let mut app = setup();

		app.add_systems(
			Update,
			execute_once(move |p| {
				p.ui_mut().insert(ui_state);
			}),
		);
		app.update();
		app.update();

		assert_state_eq!(
			&NextState::<T>::Unchanged,
			app.world().resource::<NextState<T>>()
		);
		Ok(())
	}
}
