use crate::resources::last_state::LastState;
use bevy::{prelude::*, state::state::FreelyMutableState};

impl<TState> LastState<TState>
where
	TState: FreelyMutableState,
{
	pub(crate) fn track(
		mut transition_events: EventReader<StateTransitionEvent<TState>>,
		mut last_state: ResMut<LastState<TState>>,
	) {
		for event in transition_events.read().filter(|e| e.entered != e.exited) {
			*last_state = LastState(event.exited.clone());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{ecs::world::error::TryRunScheduleError, state::app::StatesPlugin};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _State {
		A,
		B,
	}

	fn setup(state: _State) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(StateTransition, LastState::<_State>::track);
		app.add_plugins(StatesPlugin);
		app.init_resource::<LastState<_State>>();
		app.insert_state(state);

		app
	}

	#[test]
	fn set_last_state() -> Result<(), TryRunScheduleError> {
		let mut app = setup(_State::A);

		app.world_mut()
			.resource_mut::<NextState<_State>>()
			.set(_State::B);
		app.world_mut().try_run_schedule(StateTransition)?;

		assert_eq!(
			&LastState(Some(_State::A)),
			app.world().resource::<LastState<_State>>()
		);
		Ok(())
	}

	#[test]
	fn set_last_state_only_on_change() -> Result<(), TryRunScheduleError> {
		let mut app = setup(_State::A);

		app.world_mut()
			.resource_mut::<NextState<_State>>()
			.set(_State::B);
		app.world_mut().try_run_schedule(StateTransition)?;
		app.world_mut()
			.resource_mut::<NextState<_State>>()
			.set(_State::B);
		app.world_mut().try_run_schedule(StateTransition)?;

		assert_eq!(
			&LastState(Some(_State::A)),
			app.world().resource::<LastState<_State>>()
		);
		Ok(())
	}
}
