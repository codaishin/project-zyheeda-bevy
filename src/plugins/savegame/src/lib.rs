pub mod components;

mod context;
mod errors;
mod file_io;
mod resources;
mod systems;
mod traits;

use crate::{
	resources::{inspector::Inspector, unique_ids::UniqueIds},
	systems::{trigger_state::TriggerState, write_buffer::WriteBufferSystem},
};
use bevy::prelude::*;
use common::{
	components::{
		child_of_persistent::ChildOfPersistent,
		lifetime::Lifetime,
		persistent_entity::PersistentEntity,
	},
	states::{
		game_state::GameState,
		save_state::SaveState,
		transition_to_previous,
		transition_to_state,
	},
	systems::log::OnError,
	tools::action_key::{ActionKey, save_key::SaveKey},
	traits::{
		handles_input::{HandlesInput, InputSystemParam},
		handles_saving::{HandlesSaving, SavableComponent},
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use components::save::Save;
use context::SaveContext;
use file_io::FileIO;
use resources::register::Register;
use std::{
	any::{TypeId, type_name},
	marker::PhantomData,
	path::PathBuf,
	sync::{Arc, Mutex},
};

pub struct SavegamePlugin<TDependencies> {
	game_directory: PathBuf,
	_p: PhantomData<TDependencies>,
}

impl<TInput> SavegamePlugin<TInput>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
{
	pub fn from_plugin(_: &TInput) -> SavegamePluginBuilder<TInput> {
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

impl<TInput> Plugin for SavegamePlugin<TInput>
where
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
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
		let trigger_quick_save = InputSystemParam::<TInput>::trigger(
			ActionKey::Save(SaveKey::QuickSave),
			GameState::Save(SaveState::Save),
		);
		let trigger_quick_load_attempt = InputSystemParam::<TInput>::trigger(
			ActionKey::Save(SaveKey::QuickLoad),
			GameState::Save(SaveState::AttemptLoad),
		);
		let transition_to_load = transition_to_state(GameState::Save(SaveState::Load));
		let transition_to_previous = transition_to_previous::<GameState>;

		Self::register_savable_component::<Name>(app);
		Self::register_savable_component::<Transform>(app);
		Self::register_savable_component::<PersistentEntity>(app);
		Self::register_savable_component::<ChildOfPersistent>(app);
		Self::register_savable_component::<Lifetime>(app);

		app.init_resource::<Register>()
			.insert_resource(Inspector {
				quick_save: quick_save.clone(),
			})
			.add_systems(
				Startup,
				Register::update_context(quick_save.clone()).pipe(OnError::log),
			)
			.add_systems(
				Update,
				(
					trigger_quick_save,
					trigger_quick_load_attempt.run_if(Self::can_quick_load()),
				)
					.run_if(in_state(GameState::Play))
					.after(TInput::SYSTEMS),
			)
			.add_systems(
				OnEnter(GameState::Save(SaveState::Save)),
				(
					SaveContext::write_buffer_system(quick_save.clone()).pipe(OnError::log),
					SaveContext::write_file_system(quick_save.clone()).pipe(OnError::log),
				)
					.chain(),
			)
			.add_systems(
				OnEnter(GameState::Save(SaveState::AttemptLoad)),
				(
					transition_to_load.run_if(Self::can_quick_load()),
					transition_to_previous.run_if(not(Self::can_quick_load())),
				),
			)
			.add_systems(
				OnEnter(GameState::Save(SaveState::Load)),
				(
					Save::despawn_all,
					SaveContext::read_file_system(quick_save.clone()).pipe(OnError::log),
					SaveContext::read_buffer_system(quick_save).pipe(OnError::log),
				)
					.chain(),
			);
	}
}

impl<TDependencies> HandlesSaving for SavegamePlugin<TDependencies> {
	type TSaveEntityMarker = Save;

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
}

#[cfg(test)]
mod tests {
	use super::*;
	use macros::SavableComponent;
	use serde::{Deserialize, Serialize};
	use std::panic::catch_unwind;
	use testing::SingleThreadedApp;

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

		SavegamePlugin::<()>::register_savable_component::<_A>(&mut app);

		let mut expected = Register::default();
		expected.register_component::<_A>();
		assert_eq!(Some(&expected), app.world().get_resource::<Register>());
	}

	#[test]
	fn register_components() {
		let mut app = setup();

		SavegamePlugin::<()>::register_savable_component::<_A>(&mut app);
		SavegamePlugin::<()>::register_savable_component::<_B>(&mut app);

		let mut expected = Register::default();
		expected.register_component::<_A>();
		expected.register_component::<_B>();
		assert_eq!(Some(&expected), app.world().get_resource::<Register>());
	}

	#[test]
	fn crash_when_id_not_unique() {
		let result = catch_unwind(|| {
			let mut app = setup();

			SavegamePlugin::<()>::register_savable_component::<_A>(&mut app);
			SavegamePlugin::<()>::register_savable_component::<_AAgain>(&mut app);
		});

		assert!(result.is_err());
	}

	#[test]
	fn crash_when_id_not_unique_after_first_was_okay() {
		let result = catch_unwind(|| {
			let mut app = setup();

			SavegamePlugin::<()>::register_savable_component::<_B>(&mut app);
			SavegamePlugin::<()>::register_savable_component::<_A>(&mut app);
			SavegamePlugin::<()>::register_savable_component::<_AAgain>(&mut app);
		});

		assert!(result.is_err());
	}

	#[test]
	fn do_not_crash_when_same_component_registered_twice() {
		let result = catch_unwind(|| {
			let mut app = setup();

			SavegamePlugin::<()>::register_savable_component::<_A>(&mut app);
			SavegamePlugin::<()>::register_savable_component::<_A>(&mut app);
		});

		assert!(result.is_ok());
	}
}
