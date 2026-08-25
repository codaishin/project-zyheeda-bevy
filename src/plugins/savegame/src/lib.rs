mod context;
mod errors;
mod file_io;
mod resources;
mod systems;
mod traits;

use crate::{
	resources::{inspector::Inspector, unique_ids::UniqueIds},
	systems::{despawn_persistent_entities::DespawnAll, write_buffer::WriteBufferSystem},
};
use bevy::{ecs::system::ScheduleSystem, prelude::*};
use common::prelude::*;
use context::SaveContext;
use file_io::FileIO;
use resources::register::Register;
use std::{
	any::{TypeId, type_name},
	marker::PhantomData,
	path::PathBuf,
	sync::{Arc, Mutex},
};
use zyheeda_core::prelude::*;

pub struct SavegamePlugin<TDependencies> {
	game_directory: PathBuf,
	_p: PhantomData<TDependencies>,
}

impl<TGameStates> SavegamePlugin<TGameStates>
where
	TGameStates: ThreadSafe + SystemSetDefinition + HandlesGameStates,
{
	pub fn from_plugin(_: &TGameStates) -> SavegamePluginBuilder<TGameStates> {
		SavegamePluginBuilder(PhantomData)
	}
}

pub struct SavegamePluginBuilder<TDependencies>(PhantomData<TDependencies>);

impl<TDependencies> SavegamePluginBuilder<TDependencies> {
	pub fn with_game_directory(self, game_directory: PathBuf) -> SavegamePlugin<TDependencies> {
		SavegamePlugin {
			game_directory,
			_p: PhantomData,
		}
	}
}

impl<TGameStates> SavegamePlugin<TGameStates>
where
	TGameStates: ThreadSafe + SystemSetDefinition + HandlesGameStates,
{
	fn add_transitions(app: &mut App) -> Result<(), TransitionsConfigError> {
		TGameStates::add_activity_transitions(
			app,
			SettableActivity::SaveCmd,
			always,
			hash_map! {
				() => ActivityTransition::To(Activity::SaveGame(SaveGameActivity::Save)),
			},
		)?;
		TGameStates::add_activity_transitions(
			app,
			SaveGameActivity::Save,
			always,
			hash_map! {
				() => ActivityTransition::ToPreviousOf(Activity::Settable(SettableActivity::SaveCmd)),
			},
		)?;
		TGameStates::add_activity_transitions(
			app,
			SettableActivity::LoadCmd,
			Self::can_quick_load().pipe(|In(r)| Some(r)),
			hash_map! {
				true => ActivityTransition::To(Activity::SaveGame(SaveGameActivity::Load)),
				false => ActivityTransition::ToPrevious,
			},
		)?;
		TGameStates::add_activity_transitions(
			app,
			SaveGameActivity::Load,
			always,
			hash_map! {
				() => ActivityTransition::To(Activity::LoadAssets(LoadActivity::Assets)),
			},
		)?;

		Ok(())
	}
}

impl<TGameStates> Plugin for SavegamePlugin<TGameStates>
where
	TGameStates: ThreadSafe + SystemSetDefinition + HandlesGameStates,
{
	fn build(&self, app: &mut App) {
		let quick_save_file = self
			.game_directory
			.clone()
			.join("Saves")
			.join("Quick Save")
			.with_extension("json");
		let quick_save = Arc::new(Mutex::new(SaveContext::from(FileIO::with_file(
			quick_save_file,
		))));

		Self::register_savable_component::<Name>(app);
		Self::register_savable_component::<Transform>(app);
		Self::register_savable_component::<PersistentEntity>(app);
		Self::register_savable_component::<ChildOfPersistent>(app);
		Self::register_savable_component::<Lifetime>(app);

		TGameStates::add_game_state_systems(
			app,
			OnGameState::Enter(SettableActivity::SaveCmd),
			(
				SaveContext::write_buffer_system(quick_save.clone()).pipe(OnError::log),
				SaveContext::write_file_system(quick_save.clone()).pipe(OnError::log),
			)
				.in_set(ExecuteSave)
				.chain(),
		);

		TGameStates::add_game_state_systems(
			app,
			OnGameState::Enter(SettableActivity::LoadCmd),
			(
				PersistentEntity::despawn_all,
				SaveContext::read_file_system(quick_save.clone()).pipe(OnError::log),
				SaveContext::read_buffer_system(quick_save.clone()).pipe(OnError::log),
			)
				.chain(),
		);

		if let Err(err) = Self::add_transitions(app) {
			panic!("{err}");
		}

		app.init_resource::<Register>()
			.insert_resource(Inspector {
				quick_save: quick_save.clone(),
			})
			.add_systems(
				Startup,
				Register::update_context(quick_save).pipe(OnError::log),
			);
	}
}

impl<TGameStates> HandlesSaving for SavegamePlugin<TGameStates>
where
	TGameStates: ThreadSafe + AddGameStateSystem,
{
	fn can_quick_load() -> impl SystemCondition<()> {
		IntoSystem::into_system(
			Inspector::<FileIO>::quick_save_file_exists.pipe(OnError::log_and_return(|| false)),
		)
	}

	fn register_savable_component<TComponent>(app: &mut App)
	where
		TComponent: SavableComponent,
	{
		let new_type = TypeId::of::<TComponent>();
		let unique_id = TComponent::ID;

		match app.world_mut().get_resource_mut::<UniqueIds>() {
			Some(mut unique_ids) => {
				match unique_ids.0.get(&unique_id) {
					Some(old_type) if old_type != &new_type => panic!(
						"attempted to register `{}` as savable, but its id `{:?}` already exists for another component",
						type_name::<TComponent>(),
						unique_id
					),
					_ => unique_ids.0.insert(unique_id, new_type),
				};
			}
			None => {
				let unique_ids = UniqueIds::from([(unique_id, new_type)]);
				app.world_mut().insert_resource(unique_ids);
			}
		};

		match app.world_mut().get_resource_mut::<Register>() {
			None => {
				let mut register = Register::<AssetServer>::default();
				register.register_component::<TComponent>();
				app.insert_resource(register);
			}
			Some(mut register) => {
				register.register_component::<TComponent>();
			}
		}
	}

	fn on_before_save<M>(app: &mut App, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) {
		TGameStates::add_game_state_systems(
			app,
			OnGameState::Enter(SettableActivity::SaveCmd),
			systems.before(ExecuteSave),
		);
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct ExecuteSave;

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::SystemParam;
	use macros::SavableComponent;
	use serde::{Deserialize, Serialize};
	use std::panic::catch_unwind;
	use testing::SingleThreadedApp;

	struct _States;

	impl AddGameStateSystem for _States {
		fn add_game_state_systems<M, T>(
			_: &mut App,
			_: OnGameState<T>,
			_: impl IntoScheduleConfigs<ScheduleSystem, M>,
		) where
			OnGameState<T>: Into<OnGameState>,
		{
			panic!("SHOULD NOT BE USED")
		}
	}

	#[derive(SystemParam)]
	struct _StatesParam;

	#[derive(Component, SavableComponent, Serialize, Deserialize, Clone)]
	#[savable_component(id = "a")]
	struct _A;

	#[derive(Component, SavableComponent, Serialize, Deserialize, Clone)]
	#[savable_component(id = "a")]
	struct _AAgain;

	#[derive(Component, SavableComponent, Serialize, Deserialize, Clone)]
	#[savable_component(id = "b")]
	struct _B;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn register_component() {
		let mut app = setup();

		SavegamePlugin::<_States>::register_savable_component::<_A>(&mut app);

		let mut expected = Register::default();
		expected.register_component::<_A>();
		assert_eq!(Some(&expected), app.world().get_resource::<Register>());
	}

	#[test]
	fn register_components() {
		let mut app = setup();

		SavegamePlugin::<_States>::register_savable_component::<_A>(&mut app);
		SavegamePlugin::<_States>::register_savable_component::<_B>(&mut app);

		let mut expected = Register::default();
		expected.register_component::<_A>();
		expected.register_component::<_B>();
		assert_eq!(Some(&expected), app.world().get_resource::<Register>());
	}

	#[test]
	fn crash_when_id_not_unique() {
		let result = catch_unwind(|| {
			let mut app = setup();

			SavegamePlugin::<_States>::register_savable_component::<_A>(&mut app);
			SavegamePlugin::<_States>::register_savable_component::<_AAgain>(&mut app);
		});

		assert!(result.is_err());
	}

	#[test]
	fn crash_when_id_not_unique_after_first_was_okay() {
		let result = catch_unwind(|| {
			let mut app = setup();

			SavegamePlugin::<_States>::register_savable_component::<_B>(&mut app);
			SavegamePlugin::<_States>::register_savable_component::<_A>(&mut app);
			SavegamePlugin::<_States>::register_savable_component::<_AAgain>(&mut app);
		});

		assert!(result.is_err());
	}

	#[test]
	fn do_not_crash_when_same_component_registered_twice() {
		let result = catch_unwind(|| {
			let mut app = setup();

			SavegamePlugin::<_States>::register_savable_component::<_A>(&mut app);
			SavegamePlugin::<_States>::register_savable_component::<_A>(&mut app);
		});

		assert!(result.is_ok());
	}
}
