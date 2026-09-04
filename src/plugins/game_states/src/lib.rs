mod events;
mod observers;
mod plugins;
mod resources;
mod states;
mod system_params;
mod systems;

use crate::{
	events::StateEvent,
	plugins::extended_plugin::ExtendedPlugin,
	resources::{
		configured_transitions::ConfiguredTransitions,
		game_state_context::GameStateContext,
		pause_control::PauseControl,
	},
	states::state_internal::StateInternal,
	system_params::{game_states_write::GameStatesWriteParam, gui_states::GuiStates},
};
use bevy::{ecs::system::ScheduleSystem, prelude::*, state::state::StateTransitionSystems};
use common::prelude::*;
use std::{
	collections::HashMap,
	fmt::{Debug, Display},
	hash::Hash,
	marker::PhantomData,
};
use zyheeda_core::collections::ordered::OrderedSet;

pub struct GameStatesPlugin<TCommand = GameState>(PhantomData<TCommand>);

impl GameStatesPlugin {
	fn track_transitions<T>(
		previous: ResMut<PreviousStates<T>>,
		mut state_transitions: MessageReader<StateTransitionEvent<StateInternal<T>>>,
	) where
		T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
	{
		let PreviousStates(previous) = previous.into_inner();
		let last = state_transitions.read().last().and_then(|t| t.exited);

		let Some(exited) = last.and_then(StateInternal::try_into_active) else {
			return;
		};

		previous.insert(exited)
	}

	fn apply_transition_from<T>(
		command: Option<T>,
	) -> impl IntoSystem<In<Option<TransitionState<T>>>, Result<(), TransitionError<Option<T>>>, ()>
	where
		T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
	{
		#[rustfmt::skip]
		let system = move |
			In(to_state): In<Option<TransitionState<T>>>,
			mut commands: ZyheedaCommands,
			previous: Res<PreviousStates<T>>
		| {
			let Some(to_state) = to_state else {
				return Ok(());
			};
			let PreviousStates(previous) = previous.into_inner();

			match to_state {
				TransitionState::To(to_state) => {
					commands.trigger_observers_for(StateEvent::Active(to_state));
				}
				TransitionState::ToPrevious => {
					let Some(previous) = previous.last() else {
						return Err(TransitionError::NoPreviousStateFor(command));
					};

					commands.trigger_observers_for(StateEvent::Active(*previous));
				}
				TransitionState::ToPreviousOf(state) => {
					let get_previous = || {
						previous.iter().enumerate().find_map(|(i, prev)| {
							if prev != &state {
								return None;
							}

							previous.get_index(i.checked_sub(1)?)
						})
					};

					let Some(previous) = get_previous() else {
						return Err(TransitionError::NoPreviousStateFor(command));
					};

					commands.trigger_observers_for(StateEvent::Active(*previous));
				}
			};

			Ok(())
		};

		IntoSystem::into_system(system)
	}
}

impl<T> GameStatesPlugin<T>
where
	Self: TransitionUnique<T>,
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	fn configure_transitions<TResult, M>(
		app: &mut App,
		from_state: impl Into<Option<T>>,
		check: impl IntoSystem<(), Option<TResult>, M>,
		transitions: impl Into<HashMap<TResult, TransitionState<T>>>,
	) -> Result<(), TransitionsConfigError<T>>
	where
		TResult: PartialEq + Eq + Hash + ThreadSafe,
	{
		let from_state = from_state.into();
		let transitions = transitions.into();

		if let Some(from_state) = from_state {
			let any_self_transitions = transitions.values().any(
				|to_state| matches!(to_state, TransitionState::To(to_state) if to_state == &from_state),
			);
			if any_self_transitions {
				return Err(TransitionsConfigError::MayNotTransitionToSelf(from_state));
			}
		}

		if !Self::transition_unique(app, from_state) {
			return Err(TransitionsConfigError::AlreadyConfigured(from_state));
		}

		app.add_systems(
			Update,
			check
				.pipe(move |In(result)| transitions.get(&result?).copied())
				.pipe(GameStatesPlugin::apply_transition_from(from_state))
				.pipe(OnError::log)
				.run_if(StateInternal::in_state(from_state))
				.in_set(GameStateSystems),
		);

		Ok(())
	}
}

impl<T> GameStatesPlugin<GameStateExtended<T>>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	fn add_extended_plugin(app: &mut App) {
		if app.is_plugin_added::<ExtendedPlugin<T>>() {
			return;
		}

		app.add_plugins(ExtendedPlugin::<T>::default());
	}
}

impl Default for GameStatesPlugin {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl Plugin for GameStatesPlugin {
	fn build(&self, app: &mut App) {
		GuiStates::init(app);

		app.init_state::<StateInternal<GameState>>()
			.init_resource::<PreviousStates>()
			.init_resource::<GameStateContext>()
			.init_resource::<PauseControl>()
			.add_observer(StateEvent::<GameState>::set_game_state)
			.add_systems(
				StateTransition,
				(
					Self::track_transitions::<GameState>,
					GameStateContext::sync_states,
					GameStatesPlugin::game_paused().pipe(PauseControl::pause),
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

impl HandlesGameStates for GameStatesPlugin {}

impl InGameState<GameState> for GameStatesPlugin {
	fn in_game_state<const N: usize>(
		game_states: [GameState; N],
	) -> impl IntoSystem<(), bool, (), System: ReadOnlySystem> {
		IntoSystem::into_system(move |state: Option<Res<State<StateInternal<GameState>>>>| {
			let Some(state) = state else {
				return false;
			};

			match state.get().try_into_active() {
				Some(state) => game_states.contains(&state),
				None => false,
			}
		})
	}
}

impl InGameState<Gui> for GameStatesPlugin {
	fn in_game_state<const N: usize>(
		game_states: [Gui; N],
	) -> impl IntoSystem<(), bool, (), System: ReadOnlySystem> {
		IntoSystem::into_system(move |ui: Option<GuiStates>| {
			let Some(ui) = ui else {
				return false;
			};

			game_states.iter().any(|s| ui.is_on(s))
		})
	}
}

impl GameStatesWrite for GameStatesPlugin {
	type TGameStatesMut = GameStatesWriteParam<'static, 'static>;
}

impl SetToNotPause<GameState> for GameStatesPlugin {
	fn set_to_not_pause(app: &mut App, state: GameState) {
		let PauseControl { non_pause_states } = app
			.world_mut()
			.get_resource_or_init::<PauseControl>()
			.into_inner();

		non_pause_states.insert(state.into());
	}
}

impl SetToNotPause<Gui> for GameStatesPlugin {
	fn set_to_not_pause(app: &mut App, state: Gui) {
		let PauseControl { non_pause_states } = app
			.world_mut()
			.get_resource_or_init::<PauseControl>()
			.into_inner();

		non_pause_states.insert(state.into());
	}
}

impl GamePaused for GameStatesPlugin {
	fn game_paused() -> impl IntoSystem<(), bool, (), System: ReadOnlySystem> {
		IntoSystem::into_system(
			|ctx: Res<GameStateContext>, pause_control: Option<Res<PauseControl>>| {
				if ctx.game_state.try_into_active().is_none() {
					return true;
				}

				let pause_control = match pause_control {
					Some(ref pause_control) => pause_control.as_ref(),
					None => &PauseControl::default(),
				};

				ctx.into_iter()
					.any(|state| pause_control.is_pause_state(state))
			},
		)
	}
}

impl ExtendGameState for GameStatesPlugin {
	type TExtended<T>
		= GameStatesPlugin<GameStateExtended<T>>
	where
		T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy;
}

impl AddGameStateSystem<GameState> for GameStatesPlugin {
	fn add_game_state_systems<M>(
		app: &mut App,
		on_state: OnStateTransition<GameState>,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	) {
		match on_state {
			OnStateTransition::Enter(cmd) => {
				app.add_systems(OnEnter(StateInternal::active(cmd)), systems);
			}
			OnStateTransition::Exit(cmd) => {
				app.add_systems(OnExit(StateInternal::active(cmd)), systems);
			}
		}
	}
}

impl AddGameStateSystem<Gui> for GameStatesPlugin {
	fn add_game_state_systems<M>(
		app: &mut App,
		on_state: OnStateTransition<Gui>,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	) {
		match on_state {
			OnStateTransition::Enter(ui) => {
				GuiStates::on_enter(app, ui, systems);
			}
			OnStateTransition::Exit(ui) => {
				GuiStates::on_exit(app, ui, systems);
			}
		}
	}
}

impl<T> AddGameStateSystem<GameStateExtended<T>> for GameStatesPlugin<GameStateExtended<T>>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	fn add_game_state_systems<M>(
		app: &mut App,
		on_state: OnStateTransition<GameStateExtended<T>>,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	) {
		Self::add_extended_plugin(app);

		match on_state {
			OnStateTransition::Enter(cmd) => {
				app.add_systems(OnEnter(StateInternal::active(cmd)), systems);
			}
			OnStateTransition::Exit(cmd) => {
				app.add_systems(OnExit(StateInternal::active(cmd)), systems);
			}
		}
	}
}

impl TransitionUnique<GameState> for GameStatesPlugin {
	fn transition_unique(app: &mut App, state: Option<GameState>) -> bool {
		let ConfiguredTransitions(configured) = app
			.world_mut()
			.get_resource_or_init::<ConfiguredTransitions<Option<GameState>>>()
			.into_inner();

		configured.insert(state)
	}
}

impl<T> TransitionUnique<GameStateExtended<T>> for GameStatesPlugin<GameStateExtended<T>>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	fn transition_unique(app: &mut App, state: Option<GameStateExtended<T>>) -> bool {
		match state {
			None => GameStatesPlugin::<GameState>::transition_unique(app, None),
			Some(GameStateExtended::Base(state)) => {
				GameStatesPlugin::transition_unique(app, Some(state))
			}
			Some(GameStateExtended::Extended(state)) => {
				let ConfiguredTransitions(configured) = app
					.world_mut()
					.get_resource_or_init::<ConfiguredTransitions<T>>()
					.into_inner();

				configured.insert(state)
			}
		}
	}
}

impl AddActivityTransitions for GameStatesPlugin {
	fn add_activity_transitions<TResult, M>(
		app: &mut App,
		from_state: impl Into<Option<GameState>>,
		check: impl IntoSystem<(), Option<TResult>, M>,
		transitions: impl Into<HashMap<TResult, TransitionState<GameState>>>,
	) -> Result<(), TransitionsConfigError<GameState>>
	where
		TResult: PartialEq + Eq + Hash + ThreadSafe,
	{
		Self::configure_transitions(app, from_state, check, transitions)
	}
}

impl<T> AddActivityTransitions<GameStateExtended<T>> for GameStatesPlugin<GameStateExtended<T>>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	fn add_activity_transitions<TResult, M>(
		app: &mut App,
		from_state: impl Into<Option<GameStateExtended<T>>>,
		check: impl IntoSystem<(), Option<TResult>, M>,
		transitions: impl Into<HashMap<TResult, TransitionState<GameStateExtended<T>>>>,
	) -> Result<(), TransitionsConfigError<GameStateExtended<T>>>
	where
		TResult: PartialEq + Eq + Hash + ThreadSafe,
	{
		Self::add_extended_plugin(app);
		Self::configure_transitions(app, from_state, check, transitions)
	}
}

impl<TExtended> InGameState<GameStateExtended<TExtended>>
	for GameStatesPlugin<GameStateExtended<TExtended>>
where
	TExtended: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	fn in_game_state<const N: usize>(
		game_states: [GameStateExtended<TExtended>; N],
	) -> impl IntoSystem<(), bool, (), System: ReadOnlySystem> {
		IntoSystem::into_system(
			move |state: Option<Res<State<StateInternal<GameStateExtended<TExtended>>>>>| {
				let Some(state) = state else {
					return false;
				};

				let Some(state) = state.get().try_into_active() else {
					return false;
				};

				game_states.contains(&state)
			},
		)
	}
}

pub trait TransitionUnique<T> {
	fn transition_unique(app: &mut App, state: Option<T>) -> bool;
}

#[derive(Resource)]
pub(crate) struct PreviousStates<T = GameState>(OrderedSet<T>)
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy;

impl<T> Default for PreviousStates<T>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	fn default() -> Self {
		Self(OrderedSet::default())
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct GameStateSystems;

#[derive(Debug, PartialEq)]
enum TransitionError<TCommand> {
	NoPreviousStateFor(TCommand),
}

impl<TCommand> Display for TransitionError<TCommand>
where
	TCommand: Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TransitionError::NoPreviousStateFor(state) => {
				write!(f, "{state:?}: Cannot detect previous state")
			}
		}
	}
}

impl<TCommand> ErrorData for TransitionError<TCommand>
where
	TCommand: Debug + ThreadSafe,
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
	use std::collections::hash_set::HashSet;
	use testing::SingleThreadedApp;
	use zyheeda_core::prelude::*;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _Extension {
		A,
		B,
		C,
	}

	type _GameStateExtended = GameStateExtended<_Extension>;
	type _ExtendedPlugin = <GameStatesPlugin as ExtendGameState>::TExtended<_Extension>;

	const EXT_A: _GameStateExtended = _GameStateExtended::Extended(_Extension::A);
	const EXT_B: _GameStateExtended = _GameStateExtended::Extended(_Extension::B);
	const EXT_C: _GameStateExtended = _GameStateExtended::Extended(_Extension::C);

	impl From<_Extension> for _GameStateExtended {
		fn from(ext: _Extension) -> Self {
			Self::Extended(ext)
		}
	}

	mod transitions {
		use super::*;

		#[derive(Resource, Debug, PartialEq, Default)]
		struct _State {
			state: Option<StateEvent<_GameStateExtended>>,
		}

		impl _State {
			fn record(on_state: On<StateEvent<_GameStateExtended>>, mut s: ResMut<Self>) {
				s.state = Some(*on_state)
			}

			fn clear(mut s: ResMut<Self>) {
				s.state = None;
			}
		}

		fn setup() -> App {
			let mut app = App::new().single_threaded(Update);

			app.add_plugins(StatesPlugin);
			app.init_resource::<_State>();
			app.add_observer(_State::record);
			app.add_systems(First, _State::clear);

			app
		}

		#[test]
		fn apply_transition() -> Result<(), RunSystemError> {
			let mut app = setup();
			_ = _ExtendedPlugin::add_activity_transitions(
				&mut app,
				EXT_A,
				|| Some("b"),
				hash_map! {
					"b" => TransitionState::To(EXT_B),
				},
			);

			app.world_mut().run_system_once(
				|mut state: ResMut<NextState<StateInternal<_GameStateExtended>>>| {
					state.set(StateInternal::active(EXT_A));
				},
			)?;
			app.update();

			assert_eq!(
				Some(StateEvent::Active(EXT_B)),
				app.world().resource::<_State>().state
			);
			Ok(())
		}

		#[test]
		fn apply_transition_to_previous() -> Result<(), RunSystemError> {
			let mut app = setup();
			_ = _ExtendedPlugin::add_activity_transitions(
				&mut app,
				EXT_B,
				|| Some("previous"),
				hash_map! {
					"previous" => TransitionState::ToPrevious
				},
			);

			app.world_mut().run_system_once(
				|mut state: ResMut<NextState<StateInternal<_GameStateExtended>>>| {
					state.set(StateInternal::active(EXT_A));
				},
			)?;
			app.update();
			app.world_mut().run_system_once(
				|mut state: ResMut<NextState<StateInternal<_GameStateExtended>>>| {
					state.set(StateInternal::active(EXT_B));
				},
			)?;
			app.update();

			assert_eq!(
				Some(StateEvent::Active(EXT_A)),
				app.world().resource::<_State>().state
			);
			Ok(())
		}

		#[test]
		fn apply_transition_to_previous_of() -> Result<(), RunSystemError> {
			let mut app = setup();
			_ = _ExtendedPlugin::add_activity_transitions(
				&mut app,
				EXT_C,
				|| Some("previous of b"),
				hash_map! {
					"previous of b" => TransitionState::ToPreviousOf(EXT_B),
				},
			);

			app.world_mut().run_system_once(
				|mut state: ResMut<NextState<StateInternal<_GameStateExtended>>>| {
					state.set(StateInternal::active(EXT_A));
				},
			)?;
			app.update();
			app.world_mut().run_system_once(
				|mut state: ResMut<NextState<StateInternal<_GameStateExtended>>>| {
					state.set(StateInternal::active(EXT_B));
				},
			)?;
			app.update();
			app.world_mut().run_system_once(
				|mut state: ResMut<NextState<StateInternal<_GameStateExtended>>>| {
					state.set(StateInternal::active(EXT_C));
				},
			)?;
			app.update();

			assert_eq!(
				Some(StateEvent::Active(EXT_A)),
				app.world().resource::<_State>().state
			);
			Ok(())
		}

		#[test]
		fn no_transition() -> Result<(), RunSystemError> {
			let mut app = setup();
			_ = _ExtendedPlugin::add_activity_transitions(
				&mut app,
				EXT_A,
				|| Some("c"),
				hash_map! {
					"b" => TransitionState::To(EXT_B),
				},
			);

			app.world_mut().run_system_once(
				|mut state: ResMut<NextState<StateInternal<_GameStateExtended>>>| {
					state.set(StateInternal::active(EXT_A));
				},
			)?;
			app.update();

			assert_eq!(None, app.world().resource::<_State>().state);
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
			_ = _ExtendedPlugin::add_activity_transitions(
				&mut app,
				EXT_A,
				Transition::check,
				hash_map! {
					true => TransitionState::To(EXT_B),
				},
			);

			app.world_mut().run_system_once(
				|mut state: ResMut<NextState<StateInternal<_GameStateExtended>>>| {
					state.set(StateInternal::active(EXT_A));
				},
			)?;
			app.update();
			app.insert_resource(Transition(true));
			app.update();

			assert_eq!(
				Some(StateEvent::Active(EXT_B)),
				app.world().resource::<_State>().state
			);
			Ok(())
		}

		#[test]
		fn delayed_transition_to_previous() -> Result<(), RunSystemError> {
			let mut app = setup();
			_ = _ExtendedPlugin::add_activity_transitions(
				&mut app,
				EXT_B,
				Transition::check,
				hash_map! {
					true => TransitionState::ToPrevious
				},
			);

			app.world_mut().run_system_once(
				|mut state: ResMut<NextState<StateInternal<_GameStateExtended>>>| {
					state.set(StateInternal::active(EXT_A));
				},
			)?;
			app.update();
			app.world_mut().run_system_once(
				|mut state: ResMut<NextState<StateInternal<_GameStateExtended>>>| {
					state.set(StateInternal::active(EXT_B));
				},
			)?;
			app.update();
			app.update(); // <== Old transition expired by bevy
			app.insert_resource(Transition(true));
			app.update();

			assert_eq!(
				Some(StateEvent::Active(EXT_A)),
				app.world().resource::<_State>().state
			);
			Ok(())
		}

		#[test]
		fn no_transition_if_not_in_from_state() -> Result<(), RunSystemError> {
			let mut app = setup();
			_ = _ExtendedPlugin::add_activity_transitions(
				&mut app,
				EXT_A,
				always,
				hash_map! {
					() => TransitionState::To(EXT_B)
				},
			);

			app.world_mut().run_system_once(
				|mut state: ResMut<NextState<StateInternal<_GameStateExtended>>>| {
					state.set(StateInternal::active(EXT_C));
				},
			)?;
			app.update();

			assert_eq!(None, app.world().resource::<_State>().state);
			Ok(())
		}

		#[test]
		fn forbid_repeated_config() {
			let mut app = setup();
			_ = _ExtendedPlugin::add_activity_transitions(
				&mut app,
				EXT_A,
				always,
				hash_map! {
					() => TransitionState::To(EXT_B),
				},
			);

			let result = _ExtendedPlugin::add_activity_transitions(
				&mut app,
				EXT_A,
				always,
				hash_map! {
					() => TransitionState::To(EXT_B),
				},
			);

			let error = match result {
				Ok(_) => panic!("Expected Error, but was value"),
				Err(error) => error,
			};
			assert_eq!(
				TransitionsConfigError::AlreadyConfigured(Some(EXT_A)),
				error
			);
		}

		#[test]
		fn forbid_repeated_config_against_extended() {
			let mut app = setup();
			_ = GameStatesPlugin::add_activity_transitions(
				&mut app,
				GameState::Play,
				always,
				hash_map! {
					() => TransitionState::To(GameState::Pause),
				},
			);

			let result = _ExtendedPlugin::add_activity_transitions(
				&mut app,
				GameStateExtended::Base(GameState::Play),
				always,
				hash_map! {
					() => TransitionState::To(GameStateExtended::from(GameState::Pause)),
				},
			);

			let error = match result {
				Ok(_) => panic!("Expected Error, but was value"),
				Err(error) => error,
			};
			assert_eq!(
				TransitionsConfigError::AlreadyConfigured(Some(
					GameStateExtended::<_Extension>::from(GameState::Play)
				)),
				error
			);
		}

		#[test]
		fn forbid_repeated_config_against_extended_none() {
			let mut app = setup();
			_ = GameStatesPlugin::add_activity_transitions(
				&mut app,
				None,
				always,
				hash_map! {
					() => TransitionState::To(GameState::Pause),
				},
			);

			let result = _ExtendedPlugin::add_activity_transitions(
				&mut app,
				None,
				always,
				hash_map! {
					() => TransitionState::To(GameStateExtended::from(GameState::Pause)),
				},
			);

			let error = match result {
				Ok(_) => panic!("Expected Error, but was value"),
				Err(error) => error,
			};
			assert_eq!(TransitionsConfigError::AlreadyConfigured(None), error);
		}

		#[test]
		fn forbid_repeated_config_against_extended_none_reversed() {
			let mut app = setup();
			_ = _ExtendedPlugin::add_activity_transitions(
				&mut app,
				None,
				always,
				hash_map! {
					() => TransitionState::To(GameStateExtended::from(GameState::Pause)),
				},
			);

			let result = GameStatesPlugin::add_activity_transitions(
				&mut app,
				None,
				always,
				hash_map! {
					() => TransitionState::To(GameState::Pause),
				},
			);

			let error = match result {
				Ok(_) => panic!("Expected Error, but was value"),
				Err(error) => error,
			};
			assert_eq!(TransitionsConfigError::AlreadyConfigured(None), error);
		}

		#[test]
		fn forbid_transitions_to_self() {
			let mut app = setup();

			let result = _ExtendedPlugin::add_activity_transitions(
				&mut app,
				EXT_A,
				always,
				hash_map! {
					() => TransitionState::To(EXT_A),
				},
			);

			let error = match result {
				Ok(_) => panic!("Expected Error, but was value"),
				Err(error) => error,
			};
			assert_eq!(TransitionsConfigError::MayNotTransitionToSelf(EXT_A), error);
		}

		#[test]
		fn allow_transition_if_previous_broke() -> Result<(), RunSystemError> {
			let mut app = setup();
			_ = _ExtendedPlugin::add_activity_transitions(
				&mut app,
				EXT_A,
				always,
				hash_map! {
					() => TransitionState::To(EXT_A),
				},
			);
			_ = _ExtendedPlugin::add_activity_transitions(
				&mut app,
				EXT_A,
				always,
				hash_map! {
					() => TransitionState::To(EXT_B),
				},
			);

			app.world_mut().run_system_once(
				|mut state: ResMut<NextState<StateInternal<_GameStateExtended>>>| {
					state.set(StateInternal::active(EXT_A));
				},
			)?;
			app.update();

			assert_eq!(
				Some(StateEvent::Active(EXT_B)),
				app.world().resource::<_State>().state
			);
			Ok(())
		}
	}

	mod in_game_state {
		use super::*;
		use test_case::test_case;

		fn setup() -> App {
			let mut app = App::new().single_threaded(Update);

			app.add_plugins(StatesPlugin);

			app
		}

		#[test]
		fn false_if_state_missing() -> Result<(), RunSystemError> {
			let mut app = setup();

			let in_state = app
				.world_mut()
				.run_system_once(GameStatesPlugin::in_game_state([GameState::NewGame]))?;

			assert!(!in_state);
			Ok(())
		}

		#[test]
		fn true_if_state_present() -> Result<(), RunSystemError> {
			let mut app = setup();
			app.insert_state(StateInternal::active(GameState::NewGame));

			let in_state = app
				.world_mut()
				.run_system_once(GameStatesPlugin::in_game_state([GameState::NewGame]))?;

			assert!(in_state);
			Ok(())
		}

		#[test_case(StateInternal::active(GameState::Play); "other state")]
		#[test_case(StateInternal::dirty(); "dirty")]
		#[test_case(StateInternal::none(); "none")]
		fn false_if_state_does_no_match(
			state: StateInternal<GameState>,
		) -> Result<(), RunSystemError> {
			let mut app = setup();
			app.insert_state(state);

			let in_state = app
				.world_mut()
				.run_system_once(GameStatesPlugin::in_game_state([GameState::NewGame]))?;

			assert!(!in_state);
			Ok(())
		}
	}

	mod use_ui_state {
		use crate::system_params::gui_states::GuiStatesMut;

		use super::*;
		use test_case::test_case;

		fn setup() -> App {
			let mut app = App::new().single_threaded(Update);

			app.add_plugins(StatesPlugin);

			app
		}

		#[test_case(Gui::Hud; "ui")]
		#[test_case(Gui::Inventory; "inventory")]
		#[test_case(Gui::ComboOverview; "combos")]
		#[test_case(Gui::Settings; "settings")]
		fn false_if_state_missing(ui: Gui) -> Result<(), RunSystemError> {
			let mut app = setup();

			let in_state = app
				.world_mut()
				.run_system_once(GameStatesPlugin::in_game_state([ui]))?;

			assert!(!in_state);

			Ok(())
		}

		#[test_case(Gui::Hud; "ui")]
		#[test_case(Gui::Inventory; "inventory")]
		#[test_case(Gui::ComboOverview; "combos")]
		#[test_case(Gui::Settings; "settings")]
		fn true_if_state_present(ui: Gui) -> Result<(), RunSystemError> {
			let mut app = setup();
			GuiStates::init(&mut app);
			app.world_mut()
				.run_system_once(move |mut s: GuiStatesMut| {
					s.set_on(ui);
				})?;
			app.update();

			let in_state = app
				.world_mut()
				.run_system_once(GameStatesPlugin::in_game_state([ui]))?;

			assert!(in_state);

			Ok(())
		}

		#[test_case(Gui::Hud, Gui::Inventory; "ui")]
		#[test_case(Gui::Inventory, Gui::ComboOverview; "inventory")]
		#[test_case(Gui::ComboOverview, Gui::Settings; "combos")]
		#[test_case(Gui::Settings, Gui::Hud; "settings")]
		fn false_if_state_does_no_match(ui: Gui, other: Gui) -> Result<(), RunSystemError> {
			let mut app = setup();
			GuiStates::init(&mut app);
			app.world_mut()
				.run_system_once(move |mut s: GuiStatesMut| {
					s.set_on(ui);
				})?;
			app.update();

			let in_state = app
				.world_mut()
				.run_system_once(GameStatesPlugin::in_game_state([other]))?;

			assert!(!in_state);

			Ok(())
		}
	}

	mod game_paused {
		use super::*;

		fn setup<const N: usize>(
			command_state: impl Into<StateInternal<GameState>>,
			ui: [Gui; N],
		) -> App {
			let mut app = App::new().single_threaded(Update);

			app.insert_resource(GameStateContext {
				game_state: command_state.into(),
				gui: HashSet::from(ui),
			});

			app
		}

		#[test]
		fn is_paused_by_default() -> Result<(), RunSystemError> {
			let mut app = setup(StateInternal::none(), []);

			let paused = app
				.world_mut()
				.run_system_once(GameStatesPlugin::game_paused())?;

			assert!(paused);
			Ok(())
		}

		#[test]
		fn is_paused() -> Result<(), RunSystemError> {
			let mut app = setup(GameState::Pause, []);

			let paused = app
				.world_mut()
				.run_system_once(GameStatesPlugin::game_paused())?;

			assert!(paused);
			Ok(())
		}

		#[test]
		fn not_paused_on_play() -> Result<(), RunSystemError> {
			let mut app = setup(GameState::Play, []);

			let paused = app
				.world_mut()
				.run_system_once(GameStatesPlugin::game_paused())?;

			assert!(!paused);
			Ok(())
		}

		#[test]
		fn not_paused_when_activity_marked_as_not_pausing() -> Result<(), RunSystemError> {
			let mut app = setup(GameState::Pause, []);
			GameStatesPlugin::set_to_not_pause(&mut app, GameState::Pause);

			let paused = app
				.world_mut()
				.run_system_once(GameStatesPlugin::game_paused())?;

			assert!(!paused);
			Ok(())
		}

		#[test]
		fn paused_on_hud() -> Result<(), RunSystemError> {
			let mut app = setup(GameState::Play, [Gui::Hud]);

			let paused = app
				.world_mut()
				.run_system_once(GameStatesPlugin::game_paused())?;

			assert!(paused);
			Ok(())
		}

		#[test]
		fn not_paused_when_ui_marked_as_not_pausing() -> Result<(), RunSystemError> {
			let mut app = setup(GameState::Play, [Gui::Hud]);
			GameStatesPlugin::set_to_not_pause(&mut app, Gui::Hud);

			let paused = app
				.world_mut()
				.run_system_once(GameStatesPlugin::game_paused())?;

			assert!(!paused);
			Ok(())
		}
	}

	mod in_extended_state {
		use super::*;

		fn setup() -> App {
			let mut app = App::new().single_threaded(Update);

			app.add_plugins(StatesPlugin);

			app
		}

		#[test]
		fn false_if_state_missing() -> Result<(), RunSystemError> {
			let mut app = setup();

			let in_state = app
				.world_mut()
				.run_system_once(_ExtendedPlugin::in_game_state([EXT_A]))?;

			assert!(!in_state);
			Ok(())
		}

		#[test]
		fn true_if_in_state() -> Result<(), RunSystemError> {
			let mut app = setup();
			app.insert_state(StateInternal::active(EXT_A));

			let in_state = app
				.world_mut()
				.run_system_once(_ExtendedPlugin::in_game_state([EXT_A]))?;

			assert!(in_state);
			Ok(())
		}

		#[test]
		fn false_if_not_in_state() -> Result<(), RunSystemError> {
			let mut app = setup();
			app.insert_state(StateInternal::active(EXT_B));

			let in_state = app
				.world_mut()
				.run_system_once(_ExtendedPlugin::in_game_state([EXT_A]))?;

			assert!(!in_state);
			Ok(())
		}

		#[test]
		fn false_if_in_default_state() -> Result<(), RunSystemError> {
			let mut app = setup();
			app.init_state::<StateInternal<_GameStateExtended>>();

			let in_state = app
				.world_mut()
				.run_system_once(_ExtendedPlugin::in_game_state([EXT_A]))?;

			assert!(!in_state);
			Ok(())
		}

		#[test]
		fn false_if_state_dirty() -> Result<(), RunSystemError> {
			let mut app = setup();
			app.insert_state(StateInternal::<_GameStateExtended>::dirty());

			let in_state = app
				.world_mut()
				.run_system_once(_ExtendedPlugin::in_game_state([EXT_A]))?;

			assert!(!in_state);
			Ok(())
		}
	}
}
