use crate::components::agent_spawner::{AgentSpawner, SpawnerActive};
use bevy::prelude::*;
use common::traits::handles_load_tracking::Loaded;

impl AgentSpawner {
	pub(crate) fn is_loaded(loaders: Query<Option<&SpawnerActive>, With<Self>>) -> Loaded {
		Loaded(loaders.iter().all(|active| active.is_none()))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::agent_spawner::SpawnerActive;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::handles_map_generation::AgentType;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn not_loaded_when_active() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(AgentSpawner(AgentType::Player));

		let loaded = app.world_mut().run_system_once(AgentSpawner::is_loaded)?;

		assert_eq!(Loaded(false), loaded);
		Ok(())
	}

	#[test]
	fn loaded_when_inactive() -> Result<(), RunSystemError> {
		let mut app = setup();
		let mut entity = app.world_mut().spawn(AgentSpawner(AgentType::Player));
		entity.remove::<SpawnerActive>();

		let loaded = app.world_mut().run_system_once(AgentSpawner::is_loaded)?;

		assert_eq!(Loaded(true), loaded);
		Ok(())
	}

	#[test]
	fn ignore_non_spawners() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(SpawnerActive);

		let loaded = app.world_mut().run_system_once(AgentSpawner::is_loaded)?;

		assert_eq!(Loaded(true), loaded);
		Ok(())
	}
}
