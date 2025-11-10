use crate::traits::{
	current_locale::CurrentLocaleMut,
	update_current_locale::UpdateCurrentLocaleFromFolder,
};
use bevy::prelude::*;
use common::traits::{load_asset::Path, load_folder_assets::LoadFolderAssets};
use std::path::PathBuf;

impl<T> LoadRequestedAssetFolder for T where
	T: UpdateCurrentLocaleFromFolder + CurrentLocaleMut + Resource
{
}

pub(crate) trait LoadRequestedAssetFolder:
	UpdateCurrentLocaleFromFolder + CurrentLocaleMut + Resource + Sized
{
	fn load_requested_asset_folder(root_path: Path) -> impl Fn(ResMut<Self>, Res<AssetServer>) {
		load_requested_asset_folder::<Self, AssetServer>(root_path)
	}
}

fn load_requested_asset_folder<TFtlServer, TAssetServer>(
	root_path: Path,
) -> impl Fn(ResMut<TFtlServer>, Res<TAssetServer>)
where
	TFtlServer: UpdateCurrentLocaleFromFolder + CurrentLocaleMut + Resource,
	TAssetServer: LoadFolderAssets + Resource,
{
	move |mut ftl_server, asset_server| {
		if !*ftl_server.update_current_locale_from_folder() {
			return;
		};

		let locale = ftl_server.current_locale_mut();
		let ftl = locale.ln.to_string().to_lowercase();
		let file = PathBuf::from(&*root_path).join(ftl);

		locale.folder = Some(asset_server.load_folder_assets(file));

		*ftl_server.update_current_locale_from_folder() = false;
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::resources::ftl_server::Locale;
	use bevy::asset::{AssetPath, LoadedFolder};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::path::PathBuf;
	use test_case::test_case;
	use testing::{NestedMocks, SingleThreadedApp, new_handle};
	use unic_langid::{LanguageIdentifier, langid};

	#[derive(Resource, NestedMocks)]
	struct _AssetServer {
		mock: Mock_AssetServer,
	}

	#[automock]
	impl LoadFolderAssets for _AssetServer {
		fn load_folder_assets<TPath>(&self, path: TPath) -> Handle<LoadedFolder>
		where
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_folder_assets(path)
		}
	}

	#[derive(Resource)]
	struct _FtlServer {
		update: bool,
		locale: Locale,
	}

	impl UpdateCurrentLocaleFromFolder for _FtlServer {
		fn update_current_locale_from_folder(&mut self) -> &mut bool {
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
			load_requested_asset_folder::<_FtlServer, _AssetServer>(root_path),
		);
		app.insert_resource(asset_server);

		app
	}

	#[test]
	fn set_asset_folder() {
		let folder = new_handle();
		let folder_clone = folder.clone();
		let server = _AssetServer::new().with_mock(move |mock| {
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

	#[test_case(langid!("en-US"); "region")]
	#[test_case(langid!("zh-Hant"); "stript")]
	#[test_case(langid!("ja-Jpan-JP"); "complex")]
	fn use_language(ln: LanguageIdentifier) {
		let file = ln.to_string().to_lowercase();
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_load_folder_assets::<PathBuf>()
				.times(1)
				.with(eq(PathBuf::from(format!("my/path/{file}"))))
				.return_const(new_handle());
		});
		let mut app = setup(Path::from("my/path"), server);
		app.insert_resource(_FtlServer {
			update: true,
			locale: Locale {
				ln,
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
