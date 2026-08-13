mod resources;
mod states;
mod system_params;
mod systems;

use crate::{
	resources::{
		configured_transitions::ConfiguredTransitions,
		game_state_context::GameStatesContext,
	},
	states::activity::Activity,
	system_params::{
		game_states_read::GameStatesRead,
		game_states_write::GameStatesWrite,
		ui_states::UIStates,
	},
};
use bevy::{ecs::system::ScheduleSystem, prelude::*};
use common::{
	errors::{ErrorData, Level},
	systems::log::OnError,
	tools::plugin_system_set::PluginSystemSet,
	traits::{
		handles_game_states::{
			ActivityState,
			AddGameStateSystem,
			AutomaticGameStateTransitions,
			GameState,
			HandlesGameStates,
			OnGameState,
			StateTransition,
			TransitionsConfigError,
			WithOptionalTransitions,
		},
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use std::{
	collections::HashMap,
	fmt::{Debug, Display},
	hash::Hash,
};

pub struct GameStatesPlugin;

impl GameStatesPlugin {
	fn apply_transition_from(
		from_state: ActivityState,
	) -> impl IntoSystem<
		In<Option<StateTransition<ActivityState>>>,
		Result<(), TransitionError<ActivityState>>,
		(),
	> {
		#[rustfmt::skip]
		let system = move |
			In(to_state): In<Option<StateTransition<ActivityState>>>,
			mut state_transitions: MessageReader<StateTransitionEvent<Activity>>,
			mut state: ResMut<NextState<Activity>>
		| {
			let Some(to_state) = to_state else {
				return Ok(());
			};

			match to_state {
				StateTransition::To(to_state) => {
					state.set(Activity::from(to_state));
				}
				StateTransition::ToPrevious => {
					let Some(last_transition) = state_transitions.read().last() else {
						return Err(TransitionError::NoStateTransitionsFor(from_state));
					};
					let Some(ref previous) = last_transition.exited else {
						return Err(TransitionError::NoPreviousStateFor(from_state));
					};

					state.set(*previous);
				}
			};

			Ok(())
		};

		IntoSystem::into_system(system)
	}
}

impl Plugin for GameStatesPlugin {
	fn build(&self, app: &mut App) {
		UIStates::init(app);

		app.init_state::<Activity>()
			.init_resource::<GameStatesContext>()
			.add_systems(
				Update,
				GameStatesContext::sync_states.in_set(GameStateSystems),
			);
	}
}

impl SystemSetDefinition for GameStatesPlugin {
	type TSystemSet = GameStateSystems;

	const SYSTEMS: PluginSystemSet<Self::TSystemSet> = PluginSystemSet::from_set(GameStateSystems);
}

impl HandlesGameStates for GameStatesPlugin {
	type TGameStates = GameStatesRead<'static>;
	type TGameStatesMut = GameStatesWrite<'static>;
}

impl AddGameStateSystem for GameStatesPlugin {
	fn add_game_state_systems<M>(
		app: &mut App,
		on_state: OnGameState,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	) {
		let systems = systems.after(AutomaticTransitions::Override);

		match on_state {
			OnGameState::Enter(GameState::Activity(activity)) => {
				app.add_systems(OnEnter(Activity::from(activity)), systems);
			}
			OnGameState::Exit(GameState::Activity(activity)) => {
				app.add_systems(OnExit(Activity::from(activity)), systems);
			}
			OnGameState::Enter(GameState::IngameUI(ui)) => {
				UIStates::on_enter(app, ui, systems);
			}
			OnGameState::Exit(GameState::IngameUI(ui)) => {
				UIStates::on_exit(app, ui, systems);
			}
		}
	}
}

impl AutomaticGameStateTransitions<ActivityState> for GameStatesPlugin {
	type TOptionalTransitions<'a> = OptionalTransitions<'a>;

	fn automatic_game_state_transitions(
		app: &mut App,
		from_state: ActivityState,
		to_state: StateTransition<ActivityState>,
	) -> Result<Self::TOptionalTransitions<'_>, TransitionsConfigError<ActivityState>> {
		if matches!(to_state, StateTransition::To(to_state) if to_state == from_state) {
			return Err(TransitionsConfigError::MayNotTransitionToSelf(from_state));
		}

		let ConfiguredTransitions(configured_transitions) = app
			.world_mut()
			.get_resource_or_init::<ConfiguredTransitions>()
			.into_inner();
		if configured_transitions.contains(&from_state) {
			return Err(TransitionsConfigError::AlreadyConfigured(from_state));
		}

		configured_transitions.insert(from_state);

		app.configure_sets(
			OnEnter(Activity::from(from_state)),
			(
				AutomaticTransitions::Fallback,
				AutomaticTransitions::Override,
			)
				.chain(),
		);
		app.add_systems(
			OnEnter(Activity::from(from_state)),
			(move || Some(to_state))
				.pipe(GameStatesPlugin::apply_transition_from(from_state))
				.pipe(OnError::log)
				.in_set(AutomaticTransitions::Fallback),
		);

		Ok(OptionalTransitions { from_state, app })
	}
}

pub struct OptionalTransitions<'a> {
	from_state: ActivityState,
	app: &'a mut App,
}

impl WithOptionalTransitions<ActivityState> for OptionalTransitions<'_> {
	fn with_optional_transitions<TResult, M>(
		self,
		check: impl IntoSystem<(), Option<TResult>, M>,
		transitions: HashMap<TResult, StateTransition<ActivityState>>,
	) -> Result<(), TransitionsConfigError<ActivityState>>
	where
		TResult: PartialEq + Eq + Hash + ThreadSafe,
	{
		let any_self_transitions = transitions.values().any(
			|to_state| matches!(to_state, StateTransition::To(to_state) if to_state == &self.from_state),
		);
		if any_self_transitions {
			return Err(TransitionsConfigError::MayNotTransitionToSelf(
				self.from_state,
			));
		}

		self.app.add_systems(
			OnEnter(Activity::from(self.from_state)),
			check
				.pipe(move |In(result)| transitions.get(&result?).copied())
				.pipe(GameStatesPlugin::apply_transition_from(self.from_state))
				.pipe(OnError::log)
				.in_set(AutomaticTransitions::Override),
		);
		Ok(())
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum AutomaticTransitions {
	Fallback,
	Override,
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct GameStateSystems;

#[derive(Debug, PartialEq)]
enum TransitionError<T> {
	NoStateTransitionsFor(T),
	NoPreviousStateFor(T),
}

impl<T> Display for TransitionError<T>
where
	T: Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TransitionError::NoStateTransitionsFor(state) => {
				write!(
					f,
					"{state:?}: Could not retrieve state transition messages to determine previous state"
				)
			}
			TransitionError::NoPreviousStateFor(state) => {
				write!(f, "{state:?}: Cannot detect previous state")
			}
		}
	}
}

impl<T> ErrorData for TransitionError<T>
where
	T: Debug + 'static,
{
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Transition Error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		state::app::StatesPlugin,
	};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		app.init_state::<Activity>();

		app
	}

	#[test]
	fn apply_transition() -> Result<(), RunSystemError> {
		let mut app = setup();
		_ = GameStatesPlugin::automatic_game_state_transitions(
			&mut app,
			ActivityState::Paused,
			StateTransition::To(ActivityState::Play),
		);

		app.world_mut()
			.run_system_once(|mut state: ResMut<NextState<Activity>>| {
				state.set(Activity(ActivityState::Paused));
			})?;
		app.update();
		app.update();

		assert_eq!(
			&Activity(ActivityState::Play),
			app.world().resource::<State<Activity>>().get()
		);
		Ok(())
	}

	#[test]
	fn apply_transition_to_previous() -> Result<(), RunSystemError> {
		let mut app = setup();
		_ = GameStatesPlugin::automatic_game_state_transitions(
			&mut app,
			ActivityState::Paused,
			StateTransition::ToPrevious,
		);

		app.world_mut()
			.run_system_once(|mut state: ResMut<NextState<Activity>>| {
				state.set(Activity(ActivityState::Play));
			})?;
		app.update();
		app.world_mut()
			.run_system_once(|mut state: ResMut<NextState<Activity>>| {
				state.set(Activity(ActivityState::Paused));
			})?;
		app.update();
		app.update();

		assert_eq!(
			&Activity(ActivityState::Play),
			app.world().resource::<State<Activity>>().get()
		);
		Ok(())
	}

	#[test]
	fn apply_conditional_transition() -> Result<(), RunSystemError> {
		let mut app = setup();
		let transitions = GameStatesPlugin::automatic_game_state_transitions(
			&mut app,
			ActivityState::Paused,
			StateTransition::To(ActivityState::Play),
		)
		.unwrap();
		_ = transitions.with_optional_transitions(
			|| Some("new game"),
			HashMap::from([("new game", StateTransition::To(ActivityState::NewGame))]),
		);

		app.world_mut()
			.run_system_once(|mut state: ResMut<NextState<Activity>>| {
				state.set(Activity(ActivityState::Paused));
			})?;
		app.update();
		app.update();

		assert_eq!(
			&Activity(ActivityState::NewGame),
			app.world().resource::<State<Activity>>().get()
		);
		Ok(())
	}

	#[test]
	fn apply_conditional_transition_to_previous() -> Result<(), RunSystemError> {
		let mut app = setup();
		let transitions = GameStatesPlugin::automatic_game_state_transitions(
			&mut app,
			ActivityState::Paused,
			StateTransition::To(ActivityState::Play),
		)
		.unwrap();
		_ = transitions.with_optional_transitions(
			|| Some("previous"),
			HashMap::from([("previous", StateTransition::ToPrevious)]),
		);

		app.world_mut()
			.run_system_once(|mut state: ResMut<NextState<Activity>>| {
				state.set(Activity(ActivityState::StartScreen));
			})?;
		app.update();
		app.world_mut()
			.run_system_once(|mut state: ResMut<NextState<Activity>>| {
				state.set(Activity(ActivityState::Paused));
			})?;
		app.update();
		app.update();

		assert_eq!(
			&Activity(ActivityState::StartScreen),
			app.world().resource::<State<Activity>>().get()
		);
		Ok(())
	}

	#[test_case(Some("Muhaha, I'm an invalid, deal with it"); "on mismatch")]
	#[test_case(None; "on check returning none")]
	fn apply_fallback_when_conditional_transitions_do_not_match(
		result: Option<&'static str>,
	) -> Result<(), RunSystemError> {
		let mut app = setup();
		let transitions = GameStatesPlugin::automatic_game_state_transitions(
			&mut app,
			ActivityState::Paused,
			StateTransition::To(ActivityState::Play),
		)
		.unwrap();
		_ = transitions.with_optional_transitions(
			move || result,
			HashMap::from([("new game", StateTransition::To(ActivityState::NewGame))]),
		);

		app.world_mut()
			.run_system_once(|mut state: ResMut<NextState<Activity>>| {
				state.set(Activity(ActivityState::Paused));
			})?;
		app.update();
		app.update();

		assert_eq!(
			&Activity(ActivityState::Play),
			app.world().resource::<State<Activity>>().get()
		);
		Ok(())
	}

	#[test]
	fn allow_automatic_transitions_to_be_overridden() -> Result<(), RunSystemError> {
		let mut app = setup();
		GameStatesPlugin::add_game_state_systems(
			&mut app,
			OnGameState::Enter(GameState::Activity(ActivityState::Paused)),
			|mut state: ResMut<NextState<Activity>>| {
				state.set(Activity(ActivityState::StartScreen));
			},
		);
		let transitions = GameStatesPlugin::automatic_game_state_transitions(
			&mut app,
			ActivityState::Paused,
			StateTransition::To(ActivityState::Play),
		)
		.unwrap();
		_ = transitions.with_optional_transitions(
			|| Some(true),
			HashMap::from([(true, StateTransition::To(ActivityState::Save))]),
		);

		app.world_mut()
			.run_system_once(|mut state: ResMut<NextState<Activity>>| {
				state.set(Activity(ActivityState::Paused));
			})?;
		app.update();
		app.update();

		assert_eq!(
			&Activity(ActivityState::StartScreen),
			app.world().resource::<State<Activity>>().get()
		);
		Ok(())
	}

	#[test]
	fn forbid_repeated_config() {
		let mut app = setup();
		_ = GameStatesPlugin::automatic_game_state_transitions(
			&mut app,
			ActivityState::Paused,
			StateTransition::To(ActivityState::Play),
		);

		let result = GameStatesPlugin::automatic_game_state_transitions(
			&mut app,
			ActivityState::Paused,
			StateTransition::To(ActivityState::Play),
		);

		let error = match result {
			Ok(_) => panic!("Expected Error, but was value"),
			Err(error) => error,
		};
		assert_eq!(
			TransitionsConfigError::AlreadyConfigured(ActivityState::Paused),
			error
		);
	}

	#[test]
	fn forbid_transitions_to_self() {
		let mut app = setup();

		let result = GameStatesPlugin::automatic_game_state_transitions(
			&mut app,
			ActivityState::Paused,
			StateTransition::To(ActivityState::Paused),
		);

		let error = match result {
			Ok(_) => panic!("Expected Error, but was value"),
			Err(error) => error,
		};
		assert_eq!(
			TransitionsConfigError::MayNotTransitionToSelf(ActivityState::Paused),
			error
		);
	}

	#[test]
	fn forbid_transitions_to_self_in_optional_transitions() {
		let mut app = setup();

		let transitions = GameStatesPlugin::automatic_game_state_transitions(
			&mut app,
			ActivityState::Paused,
			StateTransition::To(ActivityState::Play),
		)
		.unwrap();
		let result = transitions.with_optional_transitions(
			|| Some("foo"),
			HashMap::from([
				("foo", StateTransition::ToPrevious),
				("bar", StateTransition::To(ActivityState::Paused)),
			]),
		);

		let error = match result {
			Ok(_) => panic!("Expected Error, but was value"),
			Err(error) => error,
		};
		assert_eq!(
			TransitionsConfigError::MayNotTransitionToSelf(ActivityState::Paused),
			error
		);
	}
}
