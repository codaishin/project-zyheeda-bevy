use crate::resources::SkillFolder;
use bevy::{
	ecs::system::Res,
	prelude::{Commands, Resource},
};
use common::traits::{load_asset::Path, load_folder_assets::LoadFolderAssets};

pub(crate) fn load_skills<TAssetServer: LoadFolderAssets + Resource>(
	mut commands: Commands,
	asset_server: Res<TAssetServer>,
) {
	let folder = asset_server.load_folder_assets(Path::from("skills"));
	commands.insert_resource(SkillFolder(folder));
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle, LoadedFolder},
		prelude::Resource,
	};
	use common::{test_tools::utils::SingleThreadedApp, traits::load_asset::Path};
	use mockall::{automock, predicate::eq};
	use uuid::Uuid;

	#[derive(Resource, Default)]
	struct _Server {
		mock: Mock_Server,
	}

	#[automock]
	impl LoadFolderAssets for _Server {
		fn load_folder_assets(&self, path: Path) -> Handle<LoadedFolder> {
			self.mock.load_folder_assets(path)
		}
	}

	fn setup(server: _Server) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(server);
		app.init_resource::<SkillFolder>();
		app.add_systems(Update, load_skills::<_Server>);

		app
	}

	#[test]
	fn store_folder_handle() {
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut server = _Server::default();
		server
			.mock
			.expect_load_folder_assets()
			.return_const(handle.clone());

		let mut app = setup(server);

		app.update();

		let skill_folder = app.world().resource::<SkillFolder>();

		assert_eq!(&SkillFolder(handle), skill_folder);
	}

	#[test]
	fn call_load_folder_assets_with_skill_path() {
		let mut server = _Server::default();
		server
			.mock
			.expect_load_folder_assets()
			.times(1)
			.with(eq(Path::from("skills")))
			.return_const(Handle::default());

		let mut app = setup(server);

		app.update();
	}
}
