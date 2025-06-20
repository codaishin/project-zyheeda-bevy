pub mod components;

mod context;
mod errors;
mod file_io;
mod resources;
mod systems;
mod traits;

use crate::systems::{trigger_state::TriggerState, write_buffer::WriteBufferSystem};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	components::{child_of_persistent::ChildOfPersistent, persistent_entity::PersistentEntity},
	states::game_state::GameState,
	systems::log::log,
	tools::action_key::{ActionKey, save_key::SaveKey},
	traits::{
		handles_saving::{HandlesSaving, SavableComponent},
		handles_settings::HandlesSettings,
		thread_safe::ThreadSafe,
	},
};
use components::save::Save;
use context::SaveContext;
use file_io::FileIO;
use resources::register::Register;
use std::{
	marker::PhantomData,
	path::PathBuf,
	sync::{Arc, Mutex},
};

pub struct SavegamePlugin<TDependencies> {
	game_directory: PathBuf,
	_p: PhantomData<TDependencies>,
}

impl<TSettings> SavegamePlugin<TSettings>
where
	TSettings: ThreadSafe + HandlesSettings,
{
	pub fn from_plugin(_: &TSettings) -> SavegamePluginBuilder<TSettings> {
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

impl<TSettings> Plugin for SavegamePlugin<TSettings>
where
	TSettings: ThreadSafe + HandlesSettings,
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
		let trigger_quick_save = TSettings::TKeyMap::<ActionKey>::trigger(
			ActionKey::Save(SaveKey::QuickSave),
			GameState::Saving,
		);
		let trigger_quick_load = TSettings::TKeyMap::<ActionKey>::trigger(
			ActionKey::Save(SaveKey::QuickLoad),
			GameState::LoadingSave,
		);

		Self::register_savable_component::<Name>(app);
		Self::register_savable_component::<Transform>(app);
		Self::register_savable_component::<Velocity>(app);
		Self::register_savable_component::<PersistentEntity>(app);
		Self::register_savable_component::<ChildOfPersistent>(app);

		app.init_resource::<Register>()
			.add_systems(
				Update,
				(trigger_quick_save, trigger_quick_load).run_if(in_state(GameState::Play)),
			)
			.add_systems(
				Startup,
				Register::update_context(quick_save.clone()).pipe(log),
			)
			.add_systems(
				OnEnter(GameState::Saving),
				(
					SaveContext::write_buffer_system(quick_save.clone()).pipe(log),
					SaveContext::write_file_system(quick_save.clone()).pipe(log),
				)
					.chain(),
			)
			.add_systems(
				OnEnter(GameState::LoadingSave),
				SaveContext::read_file_system(quick_save.clone()).pipe(log),
			);
	}
}

impl<TDependencies> HandlesSaving for SavegamePlugin<TDependencies> {
	type TSaveEntityMarker = Save;

	fn register_savable_component<TComponent>(app: &mut App)
	where
		TComponent: SavableComponent,
	{
		match app.world_mut().get_resource_mut::<Register>() {
			None => {
				let mut register = Register::default();
				register.register_component::<TComponent, TComponent::TDto>();
				app.insert_resource(register);
			}
			Some(mut register) => {
				register.register_component::<TComponent, TComponent::TDto>();
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;
	use serde::{Deserialize, Serialize};

	#[derive(Component, Serialize, Deserialize, Clone)]
	struct _A;

	#[derive(Component, Serialize, Deserialize, Clone)]
	struct _B;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	struct _Settings;

	#[test]
	fn register_component() {
		let mut app = setup();

		SavegamePlugin::<()>::register_savable_component::<_A>(&mut app);

		let mut expected = Register::default();
		expected.register_component::<_A, _A>();
		assert_eq!(Some(&expected), app.world().get_resource::<Register>());
	}

	#[test]
	fn register_components() {
		let mut app = setup();

		SavegamePlugin::<()>::register_savable_component::<_A>(&mut app);
		SavegamePlugin::<()>::register_savable_component::<_B>(&mut app);

		let mut expected = Register::default();
		expected.register_component::<_A, _A>();
		expected.register_component::<_B, _B>();
		assert_eq!(Some(&expected), app.world().get_resource::<Register>());
	}
}
