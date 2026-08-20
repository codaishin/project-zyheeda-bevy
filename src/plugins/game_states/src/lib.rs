mod resources;
mod states;
mod system_params;
mod systems;

use crate::{
	resources::{
		configured_transitions::ConfiguredTransitions,
		game_state_context::GameStateContext,
		game_state_roles::{GAME_STATE_ROLES_DEFAULT, GameStateRoles},
	},
	states::activity::ActivityState,
	system_params::{
		game_states_read::GameStatesRead,
		game_states_write::GameStatesWrite,
		ui_states::UIStates,
	},
};
use bevy::{ecs::system::ScheduleSystem, prelude::*, state::state::StateTransitionSystems};
use common::prelude::*;
use std::{
	collections::HashMap,
	fmt::{Debug, Display},
	hash::Hash,
};
use zyheeda_core::collections::ordered::OrderedSet;

pub struct GameStatesPlugin;

impl GameStatesPlugin {
	fn track_transitions(
		previous: ResMut<PreviousStates>,
		mut state_transitions: MessageReader<StateTransitionEvent<ActivityState>>,
	) {
		let PreviousStates(previous) = previous.into_inner();

		if let Some(Some(exited)) = state_transitions.read().last().map(|t| t.exited) {
			previous.insert(exited);
		}
	}

	fn apply_transition_from(
		from_state: Activity,
	) -> impl IntoSystem<In<Option<ActivityTransition>>, Result<(), TransitionError>, ()> {
		#[rustfmt::skip]
		let system = move |
			In(to_state): In<Option<ActivityTransition>>,
		  mut state: ResMut<NextState<ActivityState>>,
		  previous: Res<PreviousStates>
		| {
			let Some(to_state) = to_state else {
				return Ok(());
			};
			let PreviousStates(previous) = previous.into_inner();

			match to_state {
				ActivityTransition::To(to_state) => {
					state.set(ActivityState::from(to_state));
				}
				ActivityTransition::ToPrevious => {
					let Some(previous) = previous.last() else {
						return Err(TransitionError::NoPreviousStateFor(from_state));
					};

					state.set(*previous);
				}
				ActivityTransition::ToPreviousOf(activity) => {
					let get_previous = || {
						previous
							.iter()
							.enumerate()
							.find_map(|(i, ActivityState(a))| {
								if a != &activity {
									return None;
								}

								previous.get_index(i.checked_sub(1)?)
							})
					};

					let Some(previous) = get_previous() else {
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

		app.init_state::<ActivityState>()
			.init_resource::<PreviousStates>()
			.init_resource::<GameStateContext>()
			.init_resource::<GameStateRoles>()
			.add_systems(
				StateTransition,
				(
					Self::track_transitions,
					GameStateContext::sync_states,
					GameStatesPlugin::game_paused().pipe(GameStateRoles::pause),
				)
					.chain()
					.in_set(GameStateSystems)
					.after(StateTransitionSystems::EnterSchedules),
			);
	}
}

impl SystemSetDefinition for GameStatesPlugin {
	type TSystemSet = GameStateSystems;

	const SYSTEMS: PluginSystemSet<Self::TSystemSet> = PluginSystemSet::from_set(GameStateSystems);
}

impl HandlesGameStates for GameStatesPlugin {
	type TGameStates = GameStatesRead<'static>;
	type TGameStatesMut = GameStatesWrite<'static, 'static>;
}

impl AddGameStateSystem for GameStatesPlugin {
	fn add_game_state_systems<M, T>(
		app: &mut App,
		on_state: OnGameState<T>,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	) where
		OnGameState<T>: Into<OnGameState>,
	{
		match on_state.into() {
			OnGameState::Enter(GameState::Activity(activity)) => {
				app.add_systems(OnEnter(ActivityState::from(activity)), systems);
			}
			OnGameState::Exit(GameState::Activity(activity)) => {
				app.add_systems(OnExit(ActivityState::from(activity)), systems);
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

impl NonPausedStates for GameStatesPlugin {
	fn add_non_pause_state(app: &mut App, state: impl Into<GameState>) {
		let GameStateRoles { non_pause_states } = app
			.world_mut()
			.get_resource_or_init::<GameStateRoles>()
			.into_inner();

		non_pause_states.insert(state.into());
	}
}

impl AddActivityTransitions for GameStatesPlugin {
	fn add_activity_transitions<TResult, M>(
		app: &mut App,
		from_state: impl Into<Activity>,
		check: impl IntoSystem<(), Option<TResult>, M>,
		transitions: impl Into<HashMap<TResult, ActivityTransition>>,
	) -> Result<(), TransitionsConfigError>
	where
		TResult: PartialEq + Eq + Hash + ThreadSafe,
	{
		let from_state = from_state.into();
		let transitions = transitions.into();
		let any_self_transitions = transitions.values().any(
			|to_state| matches!(to_state, ActivityTransition::To(to_state) if to_state == &from_state),
		);
		if any_self_transitions {
			return Err(TransitionsConfigError::MayNotTransitionToSelf(from_state));
		}

		let ConfiguredTransitions(configured) = app
			.world_mut()
			.get_resource_or_init::<ConfiguredTransitions>()
			.into_inner();
		if configured.contains(&from_state) {
			return Err(TransitionsConfigError::AlreadyConfigured(from_state));
		}

		configured.insert(from_state);

		app.add_systems(
			Update,
			check
				.pipe(move |In(result)| transitions.get(&result?).copied())
				.pipe(GameStatesPlugin::apply_transition_from(from_state))
				.pipe(OnError::log)
				.run_if(in_state(ActivityState::from(from_state)))
				.in_set(GameStateSystems),
		);

		Ok(())
	}
}

impl GamePaused for GameStatesPlugin {
	fn game_paused() -> impl IntoSystem<(), bool, (), System: ReadOnlySystem> {
		IntoSystem::into_system(
			|states: GameStatesRead, roles: Option<Res<GameStateRoles>>| {
				let roles = match roles {
					Some(r) => r.into_inner(),
					None => &GAME_STATE_ROLES_DEFAULT,
				};

				states.iter().any(|state| roles.is_pause_state(state))
			},
		)
	}
}

#[derive(Resource, Default)]
pub(crate) struct PreviousStates(OrderedSet<ActivityState>);

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct GameStateSystems;

#[derive(Debug, PartialEq)]
enum TransitionError {
	NoPreviousStateFor(Activity),
}

impl Display for TransitionError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TransitionError::NoPreviousStateFor(state) => {
				write!(f, "{state:?}: Cannot detect previous state")
			}
		}
	}
}

impl ErrorData for TransitionError {
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
	use std::collections::hash_set::HashSet;
	use testing::SingleThreadedApp;
	use zyheeda_core::prelude::*;

	mod transitions {
		use super::*;

		fn setup() -> App {
			let mut app = App::new().single_threaded(Update);

			app.add_plugins(StatesPlugin);
			app.init_resource::<PreviousStates>();
			app.init_state::<ActivityState>();
			app.add_systems(StateTransition, GameStatesPlugin::track_transitions);

			app
		}

		#[test]
		fn apply_transition() -> Result<(), RunSystemError> {
			let mut app = setup();
			_ = GameStatesPlugin::add_activity_transitions(
				&mut app,
				SettableActivity::Paused,
				|| Some("new game"),
				hash_map! {
					"new game" => ActivityTransition::To(Activity::Settable(SettableActivity::NewGame)),
				},
			);

			app.world_mut()
				.run_system_once(|mut state: ResMut<NextState<ActivityState>>| {
					state.set(ActivityState(Activity::Settable(SettableActivity::Paused)));
				})?;
			app.update();
			app.update();

			assert_eq!(
				&ActivityState(Activity::Settable(SettableActivity::NewGame)),
				app.world().resource::<State<ActivityState>>().get()
			);
			Ok(())
		}

		#[test]
		fn apply_transition_to_previous() -> Result<(), RunSystemError> {
			let mut app = setup();
			_ = GameStatesPlugin::add_activity_transitions(
				&mut app,
				SettableActivity::Paused,
				|| Some("previous"),
				hash_map! {
					"previous" => ActivityTransition::ToPrevious
				},
			);

			app.world_mut()
				.run_system_once(|mut state: ResMut<NextState<ActivityState>>| {
					state.set(ActivityState(Activity::Settable(
						SettableActivity::StartScreen,
					)));
				})?;
			app.update();
			app.world_mut()
				.run_system_once(|mut state: ResMut<NextState<ActivityState>>| {
					state.set(ActivityState(Activity::Settable(SettableActivity::Paused)));
				})?;
			app.update();
			app.update();

			assert_eq!(
				&ActivityState(Activity::Settable(SettableActivity::StartScreen)),
				app.world().resource::<State<ActivityState>>().get()
			);
			Ok(())
		}

		#[test]
		fn apply_transition_to_previous_of() -> Result<(), RunSystemError> {
			use SettableActivity::*;

			let mut app = setup();
			_ = GameStatesPlugin::add_activity_transitions(
				&mut app,
				SettableActivity::Paused,
				|| Some("previous of new game"),
				hash_map! {
					"previous of new game" => ActivityTransition::ToPreviousOf(Activity::Settable(NewGame)),
				},
			);

			app.world_mut()
				.run_system_once(|mut state: ResMut<NextState<ActivityState>>| {
					state.set(ActivityState(Activity::Settable(StartScreen)));
				})?;
			app.update();
			app.world_mut()
				.run_system_once(|mut state: ResMut<NextState<ActivityState>>| {
					state.set(ActivityState(Activity::Settable(NewGame)));
				})?;
			app.update();
			app.world_mut()
				.run_system_once(|mut state: ResMut<NextState<ActivityState>>| {
					state.set(ActivityState(Activity::Settable(Paused)));
				})?;
			app.update();
			app.update();

			assert_eq!(
				&ActivityState(Activity::Settable(StartScreen)),
				app.world().resource::<State<ActivityState>>().get()
			);
			Ok(())
		}

		#[test]
		fn no_transition() -> Result<(), RunSystemError> {
			let mut app = setup();
			_ = GameStatesPlugin::add_activity_transitions(
				&mut app,
				SettableActivity::Paused,
				|| Some("no new game"),
				hash_map! {
					"new game" => ActivityTransition::To(Activity::Settable(SettableActivity::NewGame)),
				},
			);

			app.world_mut()
				.run_system_once(|mut state: ResMut<NextState<ActivityState>>| {
					state.set(ActivityState(Activity::Settable(SettableActivity::Paused)));
				})?;
			app.update();
			app.update();

			assert_eq!(
				&ActivityState(Activity::Settable(SettableActivity::Paused)),
				app.world().resource::<State<ActivityState>>().get()
			);
			Ok(())
		}

		#[derive(Resource)]
		struct Transition(bool);

		impl Transition {
			fn check(transition: Option<Res<Transition>>) -> Option<bool> {
				transition.map(|t| t.into_inner()).map(|Transition(t)| *t)
			}
		}

		#[test]
		fn delayed_transition() -> Result<(), RunSystemError> {
			let mut app = setup();
			_ = GameStatesPlugin::add_activity_transitions(
				&mut app,
				SettableActivity::Paused,
				Transition::check,
				hash_map! {
					true => ActivityTransition::To(Activity::Settable(SettableActivity::Play)),
				},
			);

			app.world_mut()
				.run_system_once(|mut state: ResMut<NextState<ActivityState>>| {
					state.set(ActivityState(Activity::Settable(SettableActivity::Paused)));
				})?;
			app.update();
			app.insert_resource(Transition(true));
			app.update();
			app.update();

			assert_eq!(
				&ActivityState(Activity::Settable(SettableActivity::Play)),
				app.world().resource::<State<ActivityState>>().get()
			);
			Ok(())
		}

		#[test]
		fn delayed_transition_to_previous() -> Result<(), RunSystemError> {
			let mut app = setup();
			_ = GameStatesPlugin::add_activity_transitions(
				&mut app,
				SettableActivity::Paused,
				Transition::check,
				hash_map! {
					true => ActivityTransition::ToPrevious
				},
			);

			app.world_mut()
				.run_system_once(|mut state: ResMut<NextState<ActivityState>>| {
					state.set(ActivityState(Activity::Settable(
						SettableActivity::StartScreen,
					)));
				})?;
			app.update();
			app.world_mut()
				.run_system_once(|mut state: ResMut<NextState<ActivityState>>| {
					state.set(ActivityState(Activity::Settable(SettableActivity::Paused)));
				})?;
			app.update();
			app.update(); // <== Old transition expired by bevy
			app.insert_resource(Transition(true));
			app.update();
			app.update();

			assert_eq!(
				&ActivityState(Activity::Settable(SettableActivity::StartScreen)),
				app.world().resource::<State<ActivityState>>().get()
			);
			Ok(())
		}

		#[test]
		fn no_transition_if_not_in_from_state() -> Result<(), RunSystemError> {
			let mut app = setup();
			_ = GameStatesPlugin::add_activity_transitions(
				&mut app,
				SettableActivity::Paused,
				|| Some(true),
				HashMap::from([(
					true,
					ActivityTransition::To(Activity::Settable(SettableActivity::Play)),
				)]),
			);

			app.world_mut()
				.run_system_once(|mut state: ResMut<NextState<ActivityState>>| {
					state.set(ActivityState(Activity::Settable(SettableActivity::NewGame)));
				})?;
			app.update();
			app.update();

			assert_eq!(
				&ActivityState(Activity::Settable(SettableActivity::NewGame)),
				app.world().resource::<State<ActivityState>>().get()
			);
			Ok(())
		}

		#[test]
		fn forbid_repeated_config() {
			let mut app = setup();
			_ = GameStatesPlugin::add_activity_transitions(
				&mut app,
				SettableActivity::Paused,
				|| Some(true),
				hash_map! {
					true => ActivityTransition::To(Activity::Settable(SettableActivity::Play)),
				},
			);

			let result = GameStatesPlugin::add_activity_transitions(
				&mut app,
				Activity::Settable(SettableActivity::Paused),
				|| Some(true),
				HashMap::from([(
					false,
					ActivityTransition::To(Activity::Settable(SettableActivity::NewGame)),
				)]),
			);

			let error = match result {
				Ok(_) => panic!("Expected Error, but was value"),
				Err(error) => error,
			};
			assert_eq!(
				TransitionsConfigError::AlreadyConfigured(Activity::Settable(
					SettableActivity::Paused
				)),
				error
			);
		}

		#[test]
		fn forbid_transitions_to_self() {
			let mut app = setup();

			let result = GameStatesPlugin::add_activity_transitions(
				&mut app,
				SettableActivity::Paused,
				|| Some(true),
				hash_map! {
					true => ActivityTransition::To(Activity::Settable(SettableActivity::Paused)),
				},
			);

			let error = match result {
				Ok(_) => panic!("Expected Error, but was value"),
				Err(error) => error,
			};
			assert_eq!(
				TransitionsConfigError::MayNotTransitionToSelf(Activity::Settable(
					SettableActivity::Paused
				)),
				error
			);
		}

		#[test]
		fn allow_transition_if_previous_broke() -> Result<(), RunSystemError> {
			let mut app = setup();
			_ = GameStatesPlugin::add_activity_transitions(
				&mut app,
				SettableActivity::Paused,
				|| Some(true),
				hash_map! {
					true => ActivityTransition::To(Activity::Settable(SettableActivity::Paused)),
				},
			);
			_ = GameStatesPlugin::add_activity_transitions(
				&mut app,
				SettableActivity::Paused,
				|| Some(true),
				hash_map! {
					true => ActivityTransition::To(Activity::Settable(SettableActivity::Play)),
				},
			);

			app.world_mut()
				.run_system_once(|mut state: ResMut<NextState<ActivityState>>| {
					state.set(ActivityState(Activity::Settable(SettableActivity::Paused)));
				})?;
			app.update();
			app.update();

			assert_eq!(
				&ActivityState(Activity::Settable(SettableActivity::Play)),
				app.world().resource::<State<ActivityState>>().get()
			);
			Ok(())
		}
	}

	mod game_paused {
		use super::*;

		fn setup<const N: usize>(activity: Activity, ui: [IngameUI; N]) -> App {
			let mut app = App::new().single_threaded(Update);

			app.insert_resource(GameStateContext {
				activity,
				ui: HashSet::from(ui),
			});

			app
		}

		#[test]
		fn is_paused() -> Result<(), RunSystemError> {
			let mut app = setup(Activity::Settable(SettableActivity::Paused), []);

			let paused = app
				.world_mut()
				.run_system_once(GameStatesPlugin::game_paused())?;

			assert!(paused);
			Ok(())
		}

		#[test]
		fn not_paused_on_play() -> Result<(), RunSystemError> {
			let mut app = setup(Activity::Settable(SettableActivity::Play), []);

			let paused = app
				.world_mut()
				.run_system_once(GameStatesPlugin::game_paused())?;

			assert!(!paused);
			Ok(())
		}

		#[test]
		fn not_paused_when_activity_marked_as_not_pausing() -> Result<(), RunSystemError> {
			let mut app = setup(Activity::Settable(SettableActivity::Paused), []);
			GameStatesPlugin::add_non_pause_state(&mut app, SettableActivity::Paused);

			let paused = app
				.world_mut()
				.run_system_once(GameStatesPlugin::game_paused())?;

			assert!(!paused);
			Ok(())
		}

		#[test]
		fn paused_on_hud() -> Result<(), RunSystemError> {
			let mut app = setup(Activity::Settable(SettableActivity::Play), [IngameUI::Hud]);

			let paused = app
				.world_mut()
				.run_system_once(GameStatesPlugin::game_paused())?;

			assert!(paused);
			Ok(())
		}

		#[test]
		fn not_paused_when_ui_marked_as_not_pausing() -> Result<(), RunSystemError> {
			let mut app = setup(Activity::Settable(SettableActivity::Play), [IngameUI::Hud]);
			GameStatesPlugin::add_non_pause_state(&mut app, IngameUI::Hud);

			let paused = app
				.world_mut()
				.run_system_once(GameStatesPlugin::game_paused())?;

			assert!(!paused);
			Ok(())
		}
	}
}
