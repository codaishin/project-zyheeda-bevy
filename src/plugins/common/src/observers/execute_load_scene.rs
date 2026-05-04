use crate::{
	components::load_model::{GltfSceneError, LoadModel},
	traits::accessors::get::GetMut,
	zyheeda_commands::ZyheedaCommands,
};
use bevy::prelude::*;

impl LoadModel {
	pub(crate) fn execute(
		trigger: On<Add, LoadModel>,
		mut commands: ZyheedaCommands,
		scenes: Query<&LoadModel>,
	) -> Result<(), GltfSceneError> {
		let entity = trigger.entity;

		let Ok(scene) = scenes.get(entity) else {
			return Ok(());
		};

		let Some(mut entity) = commands.get_mut(&entity) else {
			return Ok(());
		};

		match scene {
			LoadModel::GltfError(err) => return Err(*err),
			LoadModel::Scene(handle) => entity.try_insert(SceneRoot(handle.clone())),
		};

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::new_handle;

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), GltfSceneError>);

	fn setup() -> App {
		let mut app = App::new();

		app.add_observer(LoadModel::execute.pipe(|In(r), mut c: Commands| {
			c.insert_resource(_Result(r));
		}));

		app
	}

	#[test]
	fn load_asset_scene() {
		let handle = new_handle();
		let mut app = setup();

		let model = app.world_mut().spawn(LoadModel::Scene(handle.clone()));

		assert_eq!(Some(&SceneRoot(handle)), model.get::<SceneRoot>());
	}

	#[test]
	fn return_ok() {
		let mut app = setup();

		app.world_mut().spawn(LoadModel::Scene(new_handle()));

		assert_eq!(&_Result(Ok(())), app.world().resource::<_Result>(),);
	}

	#[test]
	fn return_error() {
		let mut app = setup();

		app.world_mut().spawn(LoadModel::GltfError(GltfSceneError {
			scene_count: 10,
			requested_id: 100,
		}));

		assert_eq!(
			&_Result(Err(GltfSceneError {
				scene_count: 10,
				requested_id: 100
			})),
			app.world().resource::<_Result>(),
		);
	}

	#[test]
	fn act_only_once() {
		let handle = new_handle();
		let mut app = setup();

		let mut model = app.world_mut().spawn(LoadModel::Scene(handle.clone()));
		model.insert(LoadModel::Scene(new_handle()));

		assert_eq!(Some(&SceneRoot(handle)), model.get::<SceneRoot>());
	}
}
