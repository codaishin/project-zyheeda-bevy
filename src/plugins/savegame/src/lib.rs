pub mod components;

mod context;
mod errors;
mod resources;
mod systems;
mod traits;
mod writer;

use crate::systems::buffer::BufferSystem;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	states::game_state::GameState,
	systems::log::log,
	traits::handles_saving::{HandlesSaving, SavableComponent},
};
use components::save::Save;
use context::SaveContext;
use resources::register::Register;
use std::{
	path::PathBuf,
	sync::{Arc, Mutex},
};
use writer::FileWriter;

pub struct SavegamePlugin {
	game_directory: PathBuf,
}

impl SavegamePlugin {
	pub fn from_game_directory(game_directory: PathBuf) -> Self {
		Self { game_directory }
	}
}

impl Plugin for SavegamePlugin {
	fn build(&self, app: &mut App) {
		let quick_save = self
			.game_directory
			.clone()
			.join("Saves")
			.join("Quick Save")
			.with_extension("json");
		let quick_save = FileWriter::to_destination(quick_save);
		let quick_save = Arc::new(Mutex::new(SaveContext::new(quick_save)));

		Self::register_savable_component::<Name>(app);
		Self::register_savable_component::<Transform>(app);
		Self::register_savable_component::<Velocity>(app);
		Self::register_savable_component::<PersistentEntity>(app);

		app.init_resource::<Register>()
			.add_systems(
				Startup,
				Register::update_context(quick_save.clone()).pipe(log),
			)
			.add_systems(
				OnEnter(GameState::Saving),
				(
					SaveContext::buffer_system(quick_save.clone()).pipe(log),
					SaveContext::flush_system(quick_save).pipe(log),
				)
					.chain(),
			);
	}
}

impl HandlesSaving for SavegamePlugin {
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

	#[test]
	fn register_component() {
		let mut app = setup();

		SavegamePlugin::register_savable_component::<_A>(&mut app);

		let mut expected = Register::default();
		expected.register_component::<_A, _A>();
		assert_eq!(Some(&expected), app.world().get_resource::<Register>());
	}

	#[test]
	fn register_components() {
		let mut app = setup();

		SavegamePlugin::register_savable_component::<_A>(&mut app);
		SavegamePlugin::register_savable_component::<_B>(&mut app);

		let mut expected = Register::default();
		expected.register_component::<_A, _A>();
		expected.register_component::<_B, _B>();
		assert_eq!(Some(&expected), app.world().get_resource::<Register>());
	}
}
