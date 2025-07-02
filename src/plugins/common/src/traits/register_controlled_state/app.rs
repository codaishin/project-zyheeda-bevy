use crate::{
	states::transition_to_state,
	traits::{
		automatic_transitions::{AutoTransitions, TransitionTo},
		pause_control::PauseControl,
		register_controlled_state::RegisterControlledState,
	},
};
use bevy::{
	prelude::*,
	state::{app::StatesPlugin, state::FreelyMutableState},
	time::TimePlugin,
};

impl RegisterControlledState for App {
	fn register_controlled_state<TState>(&mut self) -> &mut Self
	where
		TState: AutoTransitions + PauseControl + Default + Clone,
	{
		if !self.is_plugin_added::<StatesPlugin>() {
			self.add_plugins(StatesPlugin);
		}
		if !self.is_plugin_added::<TimePlugin>() {
			self.add_plugins(TimePlugin);
		}

		self.init_state::<TState>();

		for (trigger, transition) in TState::auto_transitions() {
			match transition {
				TransitionTo::State(state) => {
					self.add_systems(OnEnter(trigger), transition_to_state(state));
				}
				TransitionTo::PreviousState => {
					self.add_systems(OnEnter(trigger), transition_to_previous::<TState>);
				}
			}
		}

		for state in TState::non_pause_states() {
			self.add_systems(OnEnter(state.clone()), pause_virtual_time::<false>);
			self.add_systems(OnExit(state), pause_virtual_time::<true>);
		}

		self
	}
}

fn transition_to_previous<TState>(
	mut transition_events: EventReader<StateTransitionEvent<TState>>,
	mut next_state: ResMut<NextState<TState>>,
) where
	TState: FreelyMutableState + Clone,
{
	let Some(last_transition) = transition_events.read().last() else {
		return;
	};
	let Some(previous) = last_transition.exited.as_ref() else {
		return;
	};

	next_state.set(previous.clone());
}

fn pause_virtual_time<const PAUSE: bool>(mut time: ResMut<Time<Virtual>>) {
	if PAUSE {
		time.pause();
	} else {
		time.unpause();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::SingleThreadedApp;
	use bevy::ecs::world::error::TryRunScheduleError;

	#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
	enum _State {
		#[default]
		Default,
		AutoTransition,
		AutoTransitioned,
		AutoPrevious,
		Previous,
		Play,
	}

	trait SetState {
		fn set_state(&mut self, state: _State);
	}

	impl SetState for App {
		fn set_state(&mut self, state: _State) {
			self.world_mut()
				.resource_mut::<NextState<_State>>()
				.set(state);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_controlled_state::<_State>();

		app
	}

	mod required_plugins {
		use super::*;
		use crate::assert_no_panic;

		#[test]
		fn add_states_plugin() {
			let app = setup();

			assert!(app.is_plugin_added::<StatesPlugin>());
		}

		#[test]
		fn do_not_add_states_plugin_when_already_added() {
			let mut app = App::new();
			app.add_plugins(StatesPlugin);

			assert_no_panic!(app.register_controlled_state::<_State>());
		}

		#[test]
		fn add_time_plugin() {
			let app = setup();

			assert!(app.is_plugin_added::<TimePlugin>());
		}

		#[test]
		fn do_not_add_time_plugin_when_already_added() {
			let mut app = App::new();
			app.add_plugins(TimePlugin);

			assert_no_panic!(app.register_controlled_state::<_State>());
		}
	}

	mod auto_transitions {
		use super::*;

		impl AutoTransitions for _State {
			fn auto_transitions() -> impl IntoIterator<Item = (Self, TransitionTo<Self>)> {
				[
					(
						Self::AutoTransition,
						TransitionTo::State(Self::AutoTransitioned),
					),
					(Self::AutoPrevious, TransitionTo::PreviousState),
				]
			}
		}

		#[test]
		fn insert_default_state() {
			let app = setup();

			assert_eq!(
				Some(&_State::Default),
				app.world().get_resource::<State<_State>>().map(|s| s.get())
			);
		}

		#[test]
		fn auto_transition_to_next() -> Result<(), TryRunScheduleError> {
			let mut app = setup();

			app.set_state(_State::AutoTransition);
			app.world_mut().try_run_schedule(StateTransition)?;

			let next_state = app.world().resource::<NextState<_State>>();
			assert!(
				matches!(next_state, NextState::Pending(_State::AutoTransitioned)),
				"match failed
				\n expected: {:?}\
				\n      got: {:?}",
				NextState::Pending(_State::AutoTransitioned),
				next_state
			);
			Ok(())
		}

		#[test]
		fn auto_transition_to_previous() -> Result<(), TryRunScheduleError> {
			let mut app = setup();

			app.set_state(_State::Previous);
			app.world_mut().try_run_schedule(StateTransition)?;
			app.set_state(_State::AutoPrevious);
			app.world_mut().try_run_schedule(StateTransition)?;

			let next_state = app.world().resource::<NextState<_State>>();
			assert!(
				matches!(next_state, NextState::Pending(_State::Previous)),
				"match failed
				\n expected: {:?}\
				\n      got: {:?}",
				NextState::Pending(_State::Previous),
				next_state
			);
			Ok(())
		}
	}

	mod pause_control {
		use super::*;

		impl PauseControl for _State {
			fn non_pause_states() -> impl IntoIterator<Item = Self> {
				[_State::Play]
			}
		}

		#[test]
		fn pause_on_exit_non_pause_state() -> Result<(), TryRunScheduleError> {
			let mut app = setup();

			app.set_state(_State::Play);
			app.world_mut().try_run_schedule(StateTransition)?;
			app.set_state(_State::Default);
			app.world_mut().try_run_schedule(StateTransition)?;

			assert!(app.world().resource::<Time<Virtual>>().is_paused());
			Ok(())
		}

		#[test]
		fn un_pause_on_enter_non_pause_state() -> Result<(), TryRunScheduleError> {
			let mut app = setup();

			app.world_mut().resource_mut::<Time<Virtual>>().pause();
			app.set_state(_State::Play);
			app.world_mut().try_run_schedule(StateTransition)?;

			assert!(!app.world().resource::<Time<Virtual>>().is_paused());
			Ok(())
		}
	}
}
