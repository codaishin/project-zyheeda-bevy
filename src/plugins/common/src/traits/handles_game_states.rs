use crate::{
	tools::iter_helpers::{first, next},
	traits::{
		iteration::{FiniteIter, IterFinite},
		thread_safe::ThreadSafe,
	},
};
use bevy::{
	ecs::system::{ReadOnlySystemParam, ScheduleSystem, StaticSystemParam, SystemParam},
	prelude::*,
};
use macros::EnumConversions;
use std::{
	collections::{HashMap, HashSet, hash_set::Iter as HashSetIter},
	fmt::{Debug, Display},
	hash::Hash,
	ops::{Deref, DerefMut},
};

pub trait HandlesGameStates:
	AddGameStateSystem + AddActivityTransitions + SetToNotPause + GamePaused
{
	type TGameStates: ThreadSafe + for<'w, 's> ReadOnlySystemParam<Item<'w, 's>: GameStates>;
	type TGameStatesMut: ThreadSafe + for<'w, 's> SystemParam<Item<'w, 's>: GameStatesMut>;
}

pub trait AddGameStateSystem {
	fn add_game_state_systems<M, T>(
		app: &mut App,
		on_state: OnGameState<T>,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	) where
		OnGameState<T>: Into<OnGameState>;
}

pub trait AddActivityTransitions {
	fn add_activity_transitions<TResult, M>(
		app: &mut App,
		from_state: impl Into<Activity>,
		check: impl IntoSystem<(), Option<TResult>, M>,
		transitions: impl Into<HashMap<TResult, ActivityTransition>>,
	) -> Result<(), TransitionsConfigError>
	where
		TResult: PartialEq + Eq + Hash + ThreadSafe;
}

pub fn always() -> Option<()> {
	Some(())
}

pub trait InGameState: HandlesGameStates {
	fn in_game_state<const N: usize, T>(
		game_states: [T; N],
	) -> impl IntoSystem<(), bool, (), System: ReadOnlySystem>
	where
		T: Into<GameState>,
	{
		let game_states = game_states.map(|s| s.into());

		IntoSystem::into_system(move |states: StaticSystemParam<Self::TGameStates>| {
			game_states.iter().any(|game_state| match game_state {
				GameState::Activity(activity) => &states.activity() == activity,
				GameState::IngameUI(ui) => states.ui().contains(ui),
			})
		})
	}
}

impl<T> InGameState for T where T: HandlesGameStates {}

pub trait GamePaused {
	fn game_paused() -> impl IntoSystem<(), bool, (), System: ReadOnlySystem>;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ActivityTransition {
	To(Activity),
	ToPrevious,
	ToPreviousOf(Activity),
}

#[derive(Debug, PartialEq)]
pub enum TransitionsConfigError {
	AlreadyConfigured(Activity),
	MayNotTransitionToSelf(Activity),
}

impl Display for TransitionsConfigError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TransitionsConfigError::AlreadyConfigured(state) => {
				write!(f, "{state:?}: Cannot be configured more than once")
			}
			TransitionsConfigError::MayNotTransitionToSelf(state) => {
				write!(
					f,
					"{state:?}: May not automatically transition back to itself"
				)
			}
		}
	}
}

pub trait SetToNotPause {
	const DEFAULT_NON_PAUSE: &[Activity] = &[Activity::Settable(SettableActivity::Play)];

	fn set_to_not_pause(app: &mut App, state: impl Into<GameState>);
}

pub trait GameStates {
	fn activity(&self) -> Activity;
	fn ui(&self) -> &'_ HashSet<IngameUI>;
}

impl<T> GameStates for T
where
	T: Deref<Target: GameStates>,
{
	fn activity(&self) -> Activity {
		self.deref().activity()
	}

	fn ui(&self) -> &'_ HashSet<IngameUI> {
		self.deref().ui()
	}
}

pub trait IterGameStates: GameStates {
	fn iter(&self) -> GameStateIter<'_> {
		GameStateIter {
			activity: Some(self.activity()),
			ui: self.ui().iter(),
		}
	}
}

impl<T> IterGameStates for T where T: GameStates {}

pub struct GameStateIter<'a> {
	activity: Option<Activity>,
	ui: HashSetIter<'a, IngameUI>,
}

impl Iterator for GameStateIter<'_> {
	type Item = GameState;

	fn next(&mut self) -> Option<Self::Item> {
		match self.activity.take() {
			Some(activity) => Some(GameState::Activity(activity)),
			None => self.ui.next().copied().map(GameState::IngameUI),
		}
	}
}

pub trait GameStatesMut {
	type TActivitySetter<'a>: SetActivity
	where
		Self: 'a;

	#[must_use]
	fn get_activity_setter(
		&mut self,
		activity: SettableActivity,
	) -> Option<Self::TActivitySetter<'_>>;

	fn ui_mut(&mut self) -> &'_ mut HashSet<IngameUI>;
}

impl<T> GameStatesMut for T
where
	T: DerefMut<Target: GameStatesMut>,
{
	type TActivitySetter<'a>
		= <T::Target as GameStatesMut>::TActivitySetter<'a>
	where
		Self: 'a;

	fn get_activity_setter(
		&mut self,
		activity: SettableActivity,
	) -> Option<Self::TActivitySetter<'_>> {
		self.deref_mut().get_activity_setter(activity)
	}

	fn ui_mut(&mut self) -> &'_ mut HashSet<IngameUI> {
		self.deref_mut().ui_mut()
	}
}

pub trait SetActivity {
	fn set_activity(self);
}

#[derive(Debug, PartialEq)]
pub enum OnGameState<T = GameState> {
	Enter(T),
	Exit(T),
}

macro_rules! impl_on_game_state_conversions {
	($fst:ty, $($rest:ty),+ $(,)?) => {
		impl_on_game_state_conversions!($fst);
		impl_on_game_state_conversions!($($rest),+);
	};
	($ty:ty) => {
		impl From<OnGameState<$ty>> for OnGameState {
			fn from(value: OnGameState<$ty>) -> Self {
				match value {
					OnGameState::Enter(s) => OnGameState::Enter(s.into()),
					OnGameState::Exit(s) => OnGameState::Exit(s.into()),
				}
			}
		}
	};
}

impl_on_game_state_conversions!(
	Activity,
	IngameUI,
	LoadActivity,
	SettableActivity,
	SaveGameActivity,
);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumConversions)]
pub enum GameState {
	Activity(Activity),
	IngameUI(IngameUI),
}

impl IterFinite for GameState {
	fn iterator() -> FiniteIter<Self> {
		FiniteIter(first(GameState::Activity))
	}

	fn next(current: &FiniteIter<Self>) -> Option<Self> {
		use GameState::*;

		match current.0? {
			Activity(activity) => next(Activity, activity).or(first(IngameUI)),
			IngameUI(ingame_ui) => next(IngameUI, ingame_ui),
		}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumConversions)]
pub enum Activity {
	SaveGame(SaveGameActivity),
	LoadAssets(LoadActivity),
	#[enum_conversions(skip)]
	LoadDependencies(LoadActivity),
	Settable(SettableActivity),
}

impl Default for Activity {
	fn default() -> Self {
		Self::LoadAssets(LoadActivity::EssentialAssets)
	}
}

impl IterFinite for Activity {
	fn iterator() -> FiniteIter<Self> {
		FiniteIter(Some(Self::SaveGame(SaveGameActivity::Save)))
	}

	fn next(current: &FiniteIter<Self>) -> Option<Self> {
		use Activity::*;
		use LoadActivity::*;
		use SaveGameActivity::*;
		use SettableActivity::*;

		match current.0? {
			SaveGame(Save) => Some(SaveGame(Load)),
			SaveGame(Load) => Some(LoadAssets(EssentialAssets)),
			LoadAssets(EssentialAssets) => Some(LoadAssets(Assets)),
			LoadAssets(Assets) => Some(LoadDependencies(EssentialAssets)),
			LoadDependencies(EssentialAssets) => Some(LoadDependencies(Assets)),
			LoadDependencies(Assets) => Some(Settable(StartScreen)),
			Settable(StartScreen) => Some(Settable(NewGame)),
			Settable(NewGame) => Some(Settable(Play)),
			Settable(Play) => Some(Settable(Paused)),
			Settable(Paused) => Some(Settable(SaveCmd)),
			Settable(SaveCmd) => Some(Settable(LoadCmd)),
			Settable(LoadCmd) => None,
		}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SaveGameActivity {
	Save,
	Load,
}

impl From<SaveGameActivity> for GameState {
	fn from(value: SaveGameActivity) -> Self {
		Self::from(Activity::SaveGame(value))
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SettableActivity {
	StartScreen,
	NewGame,
	Play,
	Paused,
	SaveCmd,
	LoadCmd,
}

impl From<SettableActivity> for GameState {
	fn from(value: SettableActivity) -> Self {
		Self::from(Activity::Settable(value))
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum LoadActivity {
	EssentialAssets,
	Assets,
}

impl From<LoadActivity> for GameState {
	fn from(value: LoadActivity) -> Self {
		Self::from(Activity::LoadAssets(value))
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum IngameUI {
	Hud,
	Inventory,
	ComboOverview,
	Settings,
}

impl IterFinite for IngameUI {
	fn iterator() -> FiniteIter<Self> {
		FiniteIter(Some(IngameUI::Hud))
	}

	fn next(current: &FiniteIter<Self>) -> Option<Self> {
		match current.0? {
			IngameUI::Hud => Some(IngameUI::Inventory),
			IngameUI::Inventory => Some(IngameUI::ComboOverview),
			IngameUI::ComboOverview => Some(IngameUI::Settings),
			IngameUI::Settings => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	mod enum_variants {
		use super::*;

		#[test]
		fn iter_activity_states() {
			assert_eq!(
				vec![
					Activity::SaveGame(SaveGameActivity::Save),
					Activity::SaveGame(SaveGameActivity::Load),
					Activity::LoadAssets(LoadActivity::EssentialAssets),
					Activity::LoadAssets(LoadActivity::Assets),
					Activity::LoadDependencies(LoadActivity::EssentialAssets),
					Activity::LoadDependencies(LoadActivity::Assets),
					Activity::Settable(SettableActivity::StartScreen),
					Activity::Settable(SettableActivity::NewGame),
					Activity::Settable(SettableActivity::Play),
					Activity::Settable(SettableActivity::Paused),
					Activity::Settable(SettableActivity::SaveCmd),
					Activity::Settable(SettableActivity::LoadCmd),
				],
				Activity::iterator().take(100).collect::<Vec<_>>()
			);
		}

		#[test]
		fn iter_ui_states() {
			assert_eq!(
				vec![
					IngameUI::Hud,
					IngameUI::Inventory,
					IngameUI::ComboOverview,
					IngameUI::Settings,
				],
				IngameUI::iterator().take(100).collect::<Vec<_>>()
			);
		}

		#[test]
		fn iter_game_sate() {
			let activities = Activity::iterator().map(GameState::Activity);
			let uis = IngameUI::iterator().map(GameState::IngameUI);

			assert_eq!(
				activities.chain(uis).collect::<Vec<_>>(),
				GameState::iterator().take(100).collect::<Vec<_>>()
			);
		}

		#[test]
		fn iter_game_state_collection() {
			struct _GameStates {
				activity: Activity,
				ui: HashSet<IngameUI>,
			}

			impl GameStates for &'_ _GameStates {
				fn activity(&self) -> Activity {
					self.activity
				}

				fn ui(&self) -> &'_ HashSet<IngameUI> {
					&self.ui
				}
			}

			let game_states = &_GameStates {
				activity: Activity::Settable(SettableActivity::Play),
				ui: HashSet::from([IngameUI::Hud, IngameUI::Settings]),
			};

			assert_eq!(
				HashSet::from([
					GameState::Activity(Activity::Settable(SettableActivity::Play)),
					GameState::IngameUI(IngameUI::Hud),
					GameState::IngameUI(IngameUI::Settings)
				]),
				game_states.iter().collect::<HashSet<_>>()
			);
		}
	}

	mod helper_system {
		use super::*;
		use testing::SingleThreadedApp;

		struct _Plugin;

		impl HandlesGameStates for _Plugin {
			type TGameStates = _Param<'static>;
			type TGameStatesMut = _Param<'static>;
		}

		impl SetToNotPause for _Plugin {
			fn set_to_not_pause(_: &mut App, _: impl Into<GameState>) {
				panic!("NOT USED")
			}
		}

		impl AddActivityTransitions for _Plugin {
			fn add_activity_transitions<TResult, M>(
				_: &mut App,
				_: impl Into<Activity>,
				_: impl IntoSystem<(), Option<TResult>, M>,
				_: impl Into<HashMap<TResult, ActivityTransition>>,
			) -> Result<(), TransitionsConfigError>
			where
				TResult: PartialEq + Eq + Hash + ThreadSafe,
			{
				panic!("NOT USED")
			}
		}

		impl AddGameStateSystem for _Plugin {
			fn add_game_state_systems<M, T>(
				_: &mut App,
				_: OnGameState<T>,
				_: impl IntoScheduleConfigs<ScheduleSystem, M>,
			) where
				OnGameState<T>: Into<OnGameState>,
			{
				panic!("NOT USED")
			}
		}

		impl GamePaused for _Plugin {
			fn game_paused() -> impl IntoSystem<(), bool, (), System: ReadOnlySystem> {
				IntoSystem::into_system(|| panic!("NOT USED"))
			}
		}

		#[derive(Resource)]
		struct _States {
			activity: Activity,
			ui: HashSet<IngameUI>,
		}

		#[derive(SystemParam)]
		struct _Param<'w> {
			states: Res<'w, _States>,
		}

		impl GameStates for _Param<'_> {
			fn activity(&self) -> Activity {
				self.states.activity
			}

			fn ui(&self) -> &'_ HashSet<IngameUI> {
				&self.states.ui
			}
		}

		impl GameStatesMut for _Param<'_> {
			type TActivitySetter<'a>
				= _SetActivity
			where
				Self: 'a;

			fn get_activity_setter(
				&mut self,
				_: SettableActivity,
			) -> Option<Self::TActivitySetter<'_>> {
				panic!("NOT USED")
			}

			fn ui_mut(&mut self) -> &'_ mut HashSet<IngameUI> {
				panic!("NOT USED")
			}
		}

		struct _SetActivity;

		impl SetActivity for _SetActivity {
			fn set_activity(self) {
				panic!("NOT USED")
			}
		}

		#[derive(Resource, Debug, PartialEq, Default)]
		struct SystemRun(bool);

		impl SystemRun {
			fn check(run: ResMut<Self>) {
				let Self(run) = run.into_inner();

				*run = true;
			}
		}

		fn setup<const N: usize>(activity: Activity, ui: [IngameUI; N]) -> App {
			let mut app = App::new().single_threaded(Update);

			app.init_resource::<SystemRun>();
			app.insert_resource(_States {
				activity,
				ui: HashSet::from(ui),
			});

			app
		}

		#[test]
		fn run_active() {
			let mut app = setup(Activity::Settable(SettableActivity::Play), []);
			app.add_systems(
				Update,
				SystemRun::check.run_if(_Plugin::in_game_state([SettableActivity::Play])),
			);

			app.update();

			assert_eq!(&SystemRun(true), app.world().resource::<SystemRun>());
		}

		#[test]
		fn do_not_run_if_not_active() {
			let mut app = setup(Activity::Settable(SettableActivity::Paused), []);
			app.add_systems(
				Update,
				SystemRun::check.run_if(_Plugin::in_game_state([SettableActivity::Play])),
			);

			app.update();

			assert_eq!(&SystemRun(false), app.world().resource::<SystemRun>());
		}

		#[test]
		fn run_if_ui_active() {
			let mut app = setup(
				Activity::Settable(SettableActivity::Paused),
				[IngameUI::Hud],
			);
			app.add_systems(
				Update,
				SystemRun::check.run_if(_Plugin::in_game_state([IngameUI::Hud])),
			);

			app.update();

			assert_eq!(&SystemRun(true), app.world().resource::<SystemRun>());
		}

		#[test]
		fn do_not_run_ingame_ui_not_active() {
			let mut app = setup(Activity::Settable(SettableActivity::Paused), []);
			app.add_systems(
				Update,
				SystemRun::check.run_if(_Plugin::in_game_state([IngameUI::Hud])),
			);

			app.update();

			assert_eq!(&SystemRun(false), app.world().resource::<SystemRun>());
		}
	}
}
