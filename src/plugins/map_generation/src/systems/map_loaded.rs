use crate::components::map::Map;
use bevy::{prelude::*, scene::SceneInstance};
use common::{components::model::Model, traits::handles_load_tracking::Loaded};

impl Map {
	pub(crate) fn is_loaded(
		maps: Query<Option<&SceneInstance>, (With<Self>, With<Model>)>,
	) -> Loaded {
		Self::is_loaded_internal(maps)
	}

	fn is_loaded_internal<TSceneLoaded>(
		maps: Query<Option<&TSceneLoaded>, (With<Self>, With<Model>)>,
	) -> Loaded
	where
		TSceneLoaded: Component,
	{
		Loaded(maps.iter().all(|loaded| loaded.is_some()))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::components::model::Model;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _SceneLoaded;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn false_if_not_scene_loaded() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.spawn((Map::default(), Model::scene("my/path")));

		let loaded = app
			.world_mut()
			.run_system_once(Map::is_loaded_internal::<_SceneLoaded>)?;

		assert_eq!(Loaded(false), loaded);
		Ok(())
	}

	#[test]
	fn true_if_scene_loaded() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.spawn((Map::default(), Model::scene("my/path"), _SceneLoaded));

		let loaded = app
			.world_mut()
			.run_system_once(Map::is_loaded_internal::<_SceneLoaded>)?;

		assert_eq!(Loaded(true), loaded);
		Ok(())
	}

	#[test]
	fn ignore_non_map_components() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(Model::scene("my/path"));

		let loaded = app
			.world_mut()
			.run_system_once(Map::is_loaded_internal::<_SceneLoaded>)?;

		assert_eq!(Loaded(true), loaded);
		Ok(())
	}

	#[test]
	fn ignore_if_map_has_no_asset_model() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(Map::default());

		let loaded = app
			.world_mut()
			.run_system_once(Map::is_loaded_internal::<_SceneLoaded>)?;

		assert_eq!(Loaded(true), loaded);
		Ok(())
	}
}
