use crate::{
	components::{
		gltf::{GltfLookup, GltfScene},
		load_scene::{GltfSceneError, LoadScene},
	},
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::ZyheedaCommands,
};
use bevy::prelude::*;

impl GltfLookup {
	pub(crate) fn trigger_model_load(
		mut commands: ZyheedaCommands,
		assets: Res<Assets<Gltf>>,
		scenes: Query<(Entity, &GltfLookup, &GltfScene), Without<LoadScene>>,
	) {
		for (entity, GltfLookup(gltf), GltfScene(id)) in scenes {
			let Some(gltf) = assets.get(gltf) else {
				continue;
			};

			let load = match gltf.scenes.get(**id) {
				Some(scene) => LoadScene::Scene(scene.clone()),
				None => LoadScene::GltfError(GltfSceneError {
					scene_count: gltf.scenes.len(),
					requested_id: **id,
				}),
			};

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(load);
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::model::SceneId;
	use test_case::test_case;
	use testing::{SingleThreadedApp, new_handle};

	fn gltf(scenes: impl Into<Vec<Handle<Scene>>>) -> Gltf {
		Gltf {
			scenes: scenes.into(),
			named_scenes: [].into(),
			meshes: [].into(),
			named_meshes: [].into(),
			materials: [].into(),
			named_materials: [].into(),
			nodes: [].into(),
			named_nodes: [].into(),
			skins: [].into(),
			named_skins: [].into(),
			default_scene: None,
			animations: [].into(),
			named_animations: [].into(),
			source: None,
		}
	}

	fn setup<const N: usize>(models: [(&Handle<Gltf>, Gltf); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut model_assets = Assets::default();

		for (id, asset) in models {
			_ = model_assets.insert(id, asset);
		}

		app.insert_resource(model_assets);
		app.add_systems(Update, GltfLookup::trigger_model_load);

		app
	}

	#[test_case(SceneId(0); "0")]
	#[test_case(SceneId(2); "2")]
	fn load_scene(id: SceneId) {
		let scenes = [new_handle(), new_handle(), new_handle()];
		let handle = new_handle();
		let gltf = gltf(scenes.clone());
		let mut app = setup([(&handle, gltf)]);
		let entity = app
			.world_mut()
			.spawn((GltfLookup(handle), GltfScene(id)))
			.id();

		app.update();

		assert_eq!(
			Some(&LoadScene::Scene(scenes[*id].clone())),
			app.world().entity(entity).get::<LoadScene>()
		)
	}

	#[test]
	fn load_scene_error() {
		let scenes = [new_handle(), new_handle(), new_handle()];
		let handle = new_handle();
		let gltf = gltf(scenes.clone());
		let mut app = setup([(&handle, gltf)]);
		let entity = app
			.world_mut()
			.spawn((GltfLookup(handle), GltfScene(SceneId(3))))
			.id();

		app.update();

		assert_eq!(
			Some(&LoadScene::GltfError(GltfSceneError {
				scene_count: 3,
				requested_id: 3,
			})),
			app.world().entity(entity).get::<LoadScene>()
		)
	}

	#[test]
	fn do_not_load_when_load_model_already_present() {
		let current_scene = new_handle();
		let scenes = [new_handle(), new_handle(), new_handle()];
		let handle = new_handle();
		let gltf = gltf(scenes.clone());
		let mut app = setup([(&handle, gltf)]);
		let entity = app
			.world_mut()
			.spawn((
				GltfLookup(handle),
				GltfScene(SceneId(0)),
				LoadScene::Scene(current_scene.clone()),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&LoadScene::Scene(current_scene)),
			app.world().entity(entity).get::<LoadScene>()
		)
	}
}
