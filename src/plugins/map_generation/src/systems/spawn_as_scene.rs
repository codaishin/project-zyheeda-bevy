use bevy::{
	ecs::system::{Commands, In, Res, Resource},
	scene::{Scene, SceneBundle},
	transform::components::Transform,
	utils::default,
};
use common::traits::{iteration::KeyValue, load_asset::LoadAsset};

pub(crate) fn spawn_as_scene<
	TCell: KeyValue<Option<String>>,
	TAsset: LoadAsset<Scene> + Resource,
>(
	cells: In<Vec<(Transform, TCell)>>,
	mut commands: Commands,
	load_asset: Res<TAsset>,
) {
	for (transform, path) in cells.0.iter().filter_map(with_cell_path) {
		let scene = load_asset.load_asset(path);
		commands.spawn(SceneBundle {
			scene,
			transform,
			..default()
		});
	}
}

fn with_cell_path<TCell: KeyValue<Option<String>>>(
	(transform, cell): &(Transform, TCell),
) -> Option<(Transform, String)> {
	Some((*transform, cell.get_value()?))
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{AssetId, AssetPath, Handle},
		ecs::system::IntoSystem,
		utils::Uuid,
	};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Clone)]
	struct _Cell(Option<&'static str>);

	impl KeyValue<Option<String>> for _Cell {
		fn get_value(&self) -> Option<String> {
			self.0.map(|s| s.to_string())
		}
	}

	#[derive(Resource, Default)]
	struct _LoadScene(HashMap<&'static str, Handle<Scene>>);

	impl _LoadScene {
		fn new_asset(&mut self, path: &'static str) -> Handle<Scene> {
			let handle = Handle::<Scene>::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			});
			self.0.insert(path, handle.clone());

			handle
		}
	}

	impl LoadAsset<Scene> for _LoadScene {
		fn load_asset<'a, TPath: Into<AssetPath<'a>>>(&self, path: TPath) -> Handle<Scene> {
			let path: AssetPath = path.into();
			self.0
				.iter()
				.find_map(
					|(key, value)| match AssetPath::from(key.to_string()) == path {
						true => Some(value),
						false => None,
					},
				)
				.unwrap_or(&Handle::<Scene>::Weak(AssetId::Uuid {
					uuid: Uuid::new_v4(),
				}))
				.clone()
		}
	}

	fn setup(cells: Vec<(Transform, _Cell)>) -> App {
		let mut app = App::new_single_threaded([Update]);
		let return_cells = move || cells.clone();

		app.init_resource::<_LoadScene>();
		app.add_systems(
			Update,
			(return_cells).pipe(spawn_as_scene::<_Cell, _LoadScene>),
		);

		app
	}

	#[test]
	fn spawn_scene_with_transform() {
		let mut app = setup(vec![(Transform::from_xyz(1., 2., 3.), _Cell(Some("A")))]);
		let scene = app.world.resource_mut::<_LoadScene>().new_asset("A");

		app.update();

		let spawned = app
			.world
			.iter_entities()
			.find_map(|e| Some((e.get::<Handle<Scene>>()?, e.get::<Transform>()?)));

		assert_eq!(Some((&scene, &Transform::from_xyz(1., 2., 3.))), spawned);
	}
}
