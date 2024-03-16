use bevy::{
	ecs::system::{Commands, In, Res, Resource},
	scene::{Scene, SceneBundle},
	transform::components::Transform,
	utils::default,
};
use common::traits::load_asset::{LoadAsset, Path};

pub(crate) fn spawn_scene<TCell: Clone, TAsset: LoadAsset<Scene> + Resource>(
	cells: In<Vec<(Transform, TCell)>>,
	mut commands: Commands,
	load_asset: Res<TAsset>,
) where
	Path: TryFrom<TCell>,
{
	for (transform, path) in cells.0.iter().filter_map(with_cell_path) {
		let scene = load_asset.load_asset(path);
		commands.spawn(SceneBundle {
			scene,
			transform,
			..default()
		});
	}
}

fn with_cell_path<TCell: Clone>((transform, cell): &(Transform, TCell)) -> Option<(Transform, Path)>
where
	Path: TryFrom<TCell>,
{
	Some((*transform, Path::try_from(cell.clone()).ok()?))
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle},
		ecs::system::IntoSystem,
		utils::Uuid,
	};
	use common::{test_tools::utils::SingleThreadedApp, traits::load_asset::Path};
	use mockall::{automock, predicate::eq};

	#[derive(Clone)]
	struct _Cell(Option<Path>);

	impl TryFrom<_Cell> for Path {
		type Error = ();

		fn try_from(value: _Cell) -> Result<Self, Self::Error> {
			match value.0 {
				Some(path) => Ok(path),
				None => Err(()),
			}
		}
	}

	#[derive(Resource, Default)]
	struct _LoadScene {
		mock: Mock_LoadScene,
	}

	#[automock]
	impl LoadAsset<Scene> for _LoadScene {
		fn load_asset(&self, path: Path) -> Handle<Scene> {
			self.mock.load_asset(path)
		}
	}

	fn setup(cells: Vec<(Transform, _Cell)>) -> App {
		let mut app = App::new_single_threaded([Update]);
		let return_cells = move || cells.clone();

		app.init_resource::<_LoadScene>();
		app.add_systems(
			Update,
			(return_cells).pipe(spawn_scene::<_Cell, _LoadScene>),
		);

		app
	}

	#[test]
	fn spawn_scene_with_transform() {
		let scene = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup(vec![(
			Transform::from_xyz(1., 2., 3.),
			_Cell(Some(Path::from("A"))),
		)]);
		app.world
			.resource_mut::<_LoadScene>()
			.mock
			.expect_load_asset()
			.times(1)
			.with(eq(Path::from("A")))
			.return_const(scene.clone());

		app.update();

		let spawned = app
			.world
			.iter_entities()
			.find_map(|e| Some((e.get::<Handle<Scene>>()?, e.get::<Transform>()?)));

		assert_eq!(Some((&scene, &Transform::from_xyz(1., 2., 3.))), spawned);
	}
}
