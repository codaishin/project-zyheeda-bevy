use crate::{
	components::load_world_asset::{GltfSceneError, LoadWorldAsset},
	traits::accessors::get::GetMut,
	zyheeda_commands::ZyheedaCommands,
};
use bevy::prelude::*;

impl LoadWorldAsset {
	pub(crate) fn execute(
		trigger: On<Add, LoadWorldAsset>,
		mut commands: ZyheedaCommands,
		scenes: Query<&LoadWorldAsset>,
	) -> Result<(), GltfSceneError> {
		let entity = trigger.entity;

		let Ok(scene) = scenes.get(entity) else {
			return Ok(());
		};

		let Some(mut entity) = commands.get_mut(&entity) else {
			return Ok(());
		};

		match scene {
			LoadWorldAsset::GltfError(err) => return Err(*err),
			LoadWorldAsset::Scene(handle) => entity.try_insert(WorldAssetRoot(handle.clone())),
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

		app.add_observer(LoadWorldAsset::execute.pipe(|In(r), mut c: Commands| {
			c.insert_resource(_Result(r));
		}));

		app
	}

	#[test]
	fn load_asset_scene() {
		let handle = new_handle();
		let mut app = setup();

		let model = app.world_mut().spawn(LoadWorldAsset::Scene(handle.clone()));

		assert_eq!(Some(&WorldAssetRoot(handle)), model.get::<WorldAssetRoot>());
	}

	#[test]
	fn return_ok() {
		let mut app = setup();

		app.world_mut().spawn(LoadWorldAsset::Scene(new_handle()));

		assert_eq!(&_Result(Ok(())), app.world().resource::<_Result>(),);
	}

	#[test]
	fn return_error() {
		let mut app = setup();

		app.world_mut()
			.spawn(LoadWorldAsset::GltfError(GltfSceneError {
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

		let mut model = app.world_mut().spawn(LoadWorldAsset::Scene(handle.clone()));
		model.insert(LoadWorldAsset::Scene(new_handle()));

		assert_eq!(Some(&WorldAssetRoot(handle)), model.get::<WorldAssetRoot>());
	}
}
