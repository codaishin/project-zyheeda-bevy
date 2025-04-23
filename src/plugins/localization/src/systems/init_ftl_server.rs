use crate::assets::ftl::Ftl;
use bevy::{asset::LoadedFolder, prelude::*};
use common::traits::{
	load_asset::{LoadAsset, Path},
	load_folder_assets::LoadFolderAssets,
};
use std::path::PathBuf;
use unic_langid::LanguageIdentifier;

impl<T> InitFtlServer for T where
	T: From<(LanguageIdentifier, Handle<Ftl>, Handle<LoadedFolder>)> + Resource
{
}

pub(crate) trait InitFtlServer:
	From<(LanguageIdentifier, Handle<Ftl>, Handle<LoadedFolder>)> + Resource
{
	fn init_with(
		ln: LanguageIdentifier,
		root_path: Path,
	) -> impl Fn(Commands, ResMut<AssetServer>) {
		init_with::<Self, AssetServer>(ln, root_path)
	}
}

fn init_with<TFtlServer, TAssetServer>(
	ln: LanguageIdentifier,
	root_path: Path,
) -> impl Fn(Commands, ResMut<TAssetServer>)
where
	TFtlServer: From<(LanguageIdentifier, Handle<Ftl>, Handle<LoadedFolder>)> + Resource,
	TAssetServer: LoadAsset + LoadFolderAssets + Resource,
{
	move |mut commands, mut server| {
		let ftl = ln.to_string().to_lowercase();
		let file = PathBuf::from(&*root_path).join(format!("{ftl}.ftl"));
		let file = server.load_asset(file);
		let folder = PathBuf::from(&*root_path).join(ftl);
		let folder = server.load_folder_assets(folder);
		let server = TFtlServer::from((ln.clone(), file, folder));
		commands.insert_resource(server);
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use bevy::asset::AssetPath;
	use common::{
		test_tools::utils::{SingleThreadedApp, new_handle},
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::path::PathBuf;
	use unic_langid::langid;

	#[derive(Resource, NestedMocks)]
	struct _AssetServer {
		mock: Mock_AssetServer,
	}

	impl LoadAsset for _AssetServer {
		fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_asset(path)
		}
	}

	impl LoadFolderAssets for _AssetServer {
		fn load_folder_assets<TPath>(&self, path: TPath) -> Handle<LoadedFolder>
		where
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_folder_assets(path)
		}
	}

	mock! {
		_AssetServer {}
		impl LoadAsset for _AssetServer {
			fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
			where
				TAsset: Asset,
				TPath: Into<AssetPath<'static>> + 'static;
		}
		impl LoadFolderAssets for _AssetServer {
			fn load_folder_assets<TPath>(&self, path: TPath) -> Handle<LoadedFolder>
				where
					TPath: Into<AssetPath<'static>> + 'static;
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _FtlServer {
		ln: LanguageIdentifier,
		file: Handle<Ftl>,
		folder: Handle<LoadedFolder>,
	}

	impl From<(LanguageIdentifier, Handle<Ftl>, Handle<LoadedFolder>)> for _FtlServer {
		fn from(
			(ln, file, folder): (LanguageIdentifier, Handle<Ftl>, Handle<LoadedFolder>),
		) -> Self {
			Self { ln, file, folder }
		}
	}

	fn setup(ln: LanguageIdentifier, root_path: Path, asset_server: _AssetServer) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, init_with::<_FtlServer, _AssetServer>(ln, root_path));
		app.insert_resource(asset_server);

		app
	}

	#[test]
	fn init_with_language() {
		let file = new_handle::<Ftl>();
		let file_clone = file.clone();
		let folder = new_handle();
		let folder_clone = folder.clone();
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_load_asset::<Ftl, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/jp.ftl")))
				.return_const(file_clone.clone());
			mock.expect_load_folder_assets::<PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/jp")))
				.return_const(folder_clone.clone());
		});
		let mut app = setup(langid!("JP"), Path::from("my/path"), server);

		app.update();

		assert_eq!(
			Some(&_FtlServer {
				ln: langid!("JP"),
				file,
				folder
			}),
			app.world().get_resource::<_FtlServer>()
		);
	}

	#[test]
	fn init_with_language_region() {
		let file = new_handle::<Ftl>();
		let file_clone = file.clone();
		let folder = new_handle();
		let folder_clone = folder.clone();
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_load_asset::<Ftl, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/en-us.ftl")))
				.return_const(file_clone.clone());
			mock.expect_load_folder_assets::<PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/en-us")))
				.return_const(folder_clone.clone());
		});
		let mut app = setup(langid!("en-US"), Path::from("my/path"), server);

		app.update();

		assert_eq!(
			Some(&_FtlServer {
				ln: langid!("en-US"),
				file,
				folder
			}),
			app.world().get_resource::<_FtlServer>()
		);
	}

	#[test]
	fn init_with_language_script() {
		let file = new_handle::<Ftl>();
		let file_clone = file.clone();
		let folder = new_handle();
		let folder_clone = folder.clone();
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_load_asset::<Ftl, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/zh-hant.ftl")))
				.return_const(file_clone.clone());
			mock.expect_load_folder_assets::<PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/zh-hant")))
				.return_const(folder_clone.clone());
		});
		let mut app = setup(langid!("zh-Hant"), Path::from("my/path"), server);

		app.update();

		assert_eq!(
			Some(&_FtlServer {
				ln: langid!("zh-Hant"),
				file,
				folder
			}),
			app.world().get_resource::<_FtlServer>()
		);
	}

	#[test]
	fn init_with_language_complex() {
		let file = new_handle::<Ftl>();
		let file_clone = file.clone();
		let folder = new_handle();
		let folder_clone = folder.clone();
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_load_asset::<Ftl, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/ja-jpan-jp.ftl")))
				.return_const(file_clone.clone());
			mock.expect_load_folder_assets::<PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/ja-jpan-jp")))
				.return_const(folder_clone.clone());
		});
		let mut app = setup(langid!("ja-Jpan-JP"), Path::from("my/path"), server);

		app.update();

		assert_eq!(
			Some(&_FtlServer {
				ln: langid!("ja-Jpan-JP"),
				file,
				folder
			}),
			app.world().get_resource::<_FtlServer>()
		);
	}
}
