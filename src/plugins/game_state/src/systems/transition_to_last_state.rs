use crate::resources::last_state::LastState;
use bevy::{prelude::*, state::state::FreelyMutableState};

impl<TState> LastState<TState>
where
	TState: FreelyMutableState,
{
	pub(crate) fn transition(last_state: Res<Self>, mut next_state: ResMut<NextState<TState>>) {
		let Some(last_state) = last_state.0.clone() else {
			return;
		};

		next_state.set(last_state);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		state::app::StatesPlugin,
	};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _State {
		A,
		B,
	}

	fn setup(state: _State, last_state: LastState<_State>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		app.insert_resource(last_state);
		app.insert_state(state);

		app
	}

	#[test]
	fn transition_to_last_state() -> Result<(), RunSystemError> {
		let mut app = setup(_State::B, LastState(Some(_State::A)));

		app.world_mut()
			.run_system_once(LastState::<_State>::transition)?;

		let next_state = app.world().resource::<NextState<_State>>();
		assert!(
			matches!(next_state, NextState::Pending(_State::A),),
			"expected: {:?}\n     got: {:?}",
			NextState::Pending(_State::A),
			next_state
		);
		Ok(())
	}
}
