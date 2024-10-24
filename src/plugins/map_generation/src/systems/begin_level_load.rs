use crate::{components::LoadLevelCommand, map::Map, traits::SourcePath};
use bevy::{
	asset::Handle,
	ecs::system::{Commands, Resource},
	prelude::ResMut,
	reflect::TypePath,
};
use common::traits::load_asset::LoadAsset;

pub(crate) fn begin_level_load<
	TLoadMap: LoadAsset + Resource,
	TCell: SourcePath + TypePath + Sync + Send,
>(
	mut commands: Commands,
	mut map_loader: ResMut<TLoadMap>,
) {
	let map: Handle<Map<TCell>> = map_loader.load_asset(TCell::source_path());
	commands.insert_resource(LoadLevelCommand(map));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::LoadLevelCommand;
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetId, AssetPath, Handle},
		reflect::TypePath,
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{load_asset::Path, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use uuid::Uuid;

	#[derive(TypePath, Asset, Debug, PartialEq)]
	struct _Cell;

	impl SourcePath for _Cell {
		fn source_path() -> Path {
			Path::from("aaa/bbb/ccc.file_format")
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _LoadMap {
		mock: Mock_LoadMap,
	}

	#[automock]
	impl LoadAsset for _LoadMap {
		fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_asset(path)
		}
	}

	fn setup(load_map: _LoadMap) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(load_map);
		app.add_systems(Update, begin_level_load::<_LoadMap, _Cell>);

		app
	}

	#[test]
	fn insert_level_command() {
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup(_LoadMap::new().with_mock(|mock| {
			mock.expect_load_asset()
				.times(1)
				.with(eq(Path::from("aaa/bbb/ccc.file_format")))
				.return_const(handle.clone());
		}));

		app.update();

		let level_command = app.world().get_resource::<LoadLevelCommand<_Cell>>();

		assert_eq!(Some(&LoadLevelCommand(handle)), level_command);
	}
}
