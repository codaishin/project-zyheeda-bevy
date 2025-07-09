use crate::traits::{current_locale::CurrentLocaleMut, requested_language::UpdateCurrentLocaleMut};
use bevy::prelude::*;
use common::traits::{
	load_asset::{AssetNotFound, Path, TryLoadAsset},
	load_folder_assets::LoadFolderAssets,
};
use std::path::PathBuf;

impl<T> LoadRequestedAssets for T where T: UpdateCurrentLocaleMut + CurrentLocaleMut + Resource {}

pub(crate) trait LoadRequestedAssets:
	UpdateCurrentLocaleMut + CurrentLocaleMut + Resource + Sized
{
	fn load_requested_assets(root_path: Path) -> impl Fn(ResMut<Self>, ResMut<AssetServer>) {
		load_requested_assets::<Self, AssetServer>(root_path)
	}
}

fn load_requested_assets<TFtlServer, TAssetServer>(
	root_path: Path,
) -> impl Fn(ResMut<TFtlServer>, ResMut<TAssetServer>)
where
	TFtlServer: UpdateCurrentLocaleMut + CurrentLocaleMut + Resource,
	TAssetServer: TryLoadAsset + LoadFolderAssets + Resource,
{
	move |mut ftl_server, mut asset_server| {
		if !*ftl_server.update_current_locale() {
			return;
		};

		let locale = ftl_server.current_locale_mut();
		let ftl = locale.ln.to_string().to_lowercase();
		let file = PathBuf::from(&*root_path).join(format!("{ftl}.ftl"));

		match asset_server.try_load_asset(file) {
			Ok(file) => locale.file = Some(file),
			Err(AssetNotFound) => {
				let folder = PathBuf::from(&*root_path).join(ftl);
				let folder = asset_server.load_folder_assets(folder);
				locale.folder = Some(folder);
			}
		}

		*ftl_server.update_current_locale() = false;
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{assets::ftl::Ftl, resources::ftl_server::Locale};
	use bevy::asset::{AssetPath, LoadedFolder};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::path::PathBuf;
	use testing::{NestedMocks, SingleThreadedApp, new_handle};
	use unic_langid::langid;

	#[derive(Resource, NestedMocks)]
	struct _AssetServer {
		mock: Mock_AssetServer,
	}

	impl TryLoadAsset for _AssetServer {
		fn try_load_asset<TAsset, TPath>(
			&mut self,
			path: TPath,
		) -> Result<Handle<TAsset>, AssetNotFound>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.try_load_asset(path)
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
		impl TryLoadAsset for _AssetServer {
			fn try_load_asset<TAsset, TPath>(
			&mut self,
			path: TPath,
		) -> Result<Handle<TAsset>, AssetNotFound>
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

	#[derive(Resource)]
	struct _FtlServer {
		update: bool,
		locale: Locale,
	}

	impl UpdateCurrentLocaleMut for _FtlServer {
		fn update_current_locale(&mut self) -> &mut bool {
			&mut self.update
		}
	}

	impl CurrentLocaleMut for _FtlServer {
		fn current_locale_mut(&mut self) -> &mut Locale {
			&mut self.locale
		}
	}

	fn setup(root_path: Path, asset_server: _AssetServer) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			load_requested_assets::<_FtlServer, _AssetServer>(root_path),
		);
		app.insert_resource(asset_server);

		app
	}

	#[test]
	fn set_asset_file() {
		let file = new_handle::<Ftl>();
		let file_clone = file.clone();
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_try_load_asset::<Ftl, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/jp.ftl")))
				.return_const(Ok(file_clone.clone()));
			mock.expect_load_folder_assets::<PathBuf>()
				.never()
				.return_const(new_handle());
		});
		let mut app = setup(Path::from("my/path"), server);
		app.insert_resource(_FtlServer {
			update: true,
			locale: Locale {
				ln: langid!("JP"),
				file: None,
				folder: None,
				bundle: None,
			},
		});

		app.update();

		let server = app.world().resource::<_FtlServer>();
		assert_eq!(
			(&Some(file), &None, false),
			(&server.locale.file, &server.locale.folder, server.update)
		);
	}

	#[test]
	fn set_asset_folder() {
		let folder = new_handle::<LoadedFolder>();
		let folder_clone = folder.clone();
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_try_load_asset::<Ftl, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/jp.ftl")))
				.return_const(Err(AssetNotFound));
			mock.expect_load_folder_assets::<PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/jp")))
				.return_const(folder_clone.clone());
		});
		let mut app = setup(Path::from("my/path"), server);
		app.insert_resource(_FtlServer {
			update: true,
			locale: Locale {
				ln: langid!("JP"),
				file: None,
				folder: None,
				bundle: None,
			},
		});

		app.update();

		let server = app.world().resource::<_FtlServer>();
		assert_eq!(
			(&None, &Some(folder), false),
			(&server.locale.file, &server.locale.folder, server.update)
		);
	}

	#[test]
	fn use_language_region() {
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_try_load_asset::<Ftl, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/en-us.ftl")))
				.return_const(Err(AssetNotFound));
			mock.expect_load_folder_assets::<PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/en-us")))
				.return_const(new_handle());
		});
		let mut app = setup(Path::from("my/path"), server);
		app.insert_resource(_FtlServer {
			update: true,
			locale: Locale {
				ln: langid!("en-US"),
				file: None,
				folder: None,
				bundle: None,
			},
		});

		app.update();
	}

	#[test]
	fn use_language_script() {
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_try_load_asset::<Ftl, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/zh-hant.ftl")))
				.return_const(Err(AssetNotFound));
			mock.expect_load_folder_assets::<PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/zh-hant")))
				.return_const(new_handle());
		});
		let mut app = setup(Path::from("my/path"), server);
		app.insert_resource(_FtlServer {
			update: true,
			locale: Locale {
				ln: langid!("zh-Hant"),
				file: None,
				folder: None,
				bundle: None,
			},
		});

		app.update();
	}

	#[test]
	fn use_language_complex() {
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_try_load_asset::<Ftl, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/ja-jpan-jp.ftl")))
				.return_const(Err(AssetNotFound));
			mock.expect_load_folder_assets::<PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/ja-jpan-jp")))
				.return_const(new_handle());
		});
		let mut app = setup(Path::from("my/path"), server);
		app.insert_resource(_FtlServer {
			update: true,
			locale: Locale {
				ln: langid!("ja-Jpan-JP"),
				file: None,
				folder: None,
				bundle: None,
			},
		});

		app.update();
	}

	#[test]
	fn do_nothing_when_not_set_for_update() {
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_try_load_asset::<Ftl, PathBuf>()
				.never()
				.return_const(Err(AssetNotFound));
			mock.expect_load_folder_assets::<PathBuf>()
				.never()
				.return_const(new_handle());
		});
		let mut app = setup(Path::from("my/path"), server);
		app.insert_resource(_FtlServer {
			update: false,
			locale: Locale {
				ln: langid!("JP"),
				file: None,
				folder: None,
				bundle: None,
			},
		});

		app.update();

		let server = app.world().resource::<_FtlServer>();
		assert_eq!(
			(&None, &None, false),
			(&server.locale.file, &server.locale.folder, server.update)
		);
	}
}
