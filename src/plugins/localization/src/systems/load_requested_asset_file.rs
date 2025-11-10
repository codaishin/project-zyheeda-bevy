use crate::traits::{
	current_locale::CurrentLocaleMut,
	update_current_locale::UpdateCurrentLocaleFromFile,
};
use bevy::prelude::*;
use common::traits::load_asset::{LoadAsset, Path};
use std::path::PathBuf;

impl<T> LoadRequestedAssetFile for T where
	T: UpdateCurrentLocaleFromFile + CurrentLocaleMut + Resource
{
}

pub(crate) trait LoadRequestedAssetFile:
	UpdateCurrentLocaleFromFile + CurrentLocaleMut + Resource + Sized
{
	fn load_requested_asset_file(root_path: Path) -> impl Fn(ResMut<Self>, ResMut<AssetServer>) {
		load_requested_asset_file::<Self, AssetServer>(root_path)
	}
}

fn load_requested_asset_file<TFtlServer, TAssetServer>(
	root_path: Path,
) -> impl Fn(ResMut<TFtlServer>, ResMut<TAssetServer>)
where
	TFtlServer: UpdateCurrentLocaleFromFile + CurrentLocaleMut + Resource,
	TAssetServer: LoadAsset + Resource,
{
	move |mut ftl_server, mut asset_server| {
		if !*ftl_server.update_current_locale_from_file() {
			return;
		};

		let locale = ftl_server.current_locale_mut();
		let ftl = locale.ln.to_string().to_lowercase();
		let file = PathBuf::from(&*root_path).join(format!("{ftl}.ftl"));

		locale.file = Some(asset_server.load_asset(file));

		*ftl_server.update_current_locale_from_file() = false;
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{assets::ftl::Ftl, resources::ftl_server::Locale};
	use bevy::asset::AssetPath;
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
	impl LoadAsset for _AssetServer {
		fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_asset(path)
		}
	}

	#[derive(Resource)]
	struct _FtlServer {
		update: bool,
		locale: Locale,
	}

	impl UpdateCurrentLocaleFromFile for _FtlServer {
		fn update_current_locale_from_file(&mut self) -> &mut bool {
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
			load_requested_asset_file::<_FtlServer, _AssetServer>(root_path),
		);
		app.insert_resource(asset_server);

		app
	}

	#[test]
	fn set_asset_file() {
		let file = new_handle::<Ftl>();
		let file_clone = file.clone();
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_load_asset::<Ftl, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/jp.ftl")))
				.return_const(file_clone.clone());
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

	#[test_case(langid!("en-US"); "region")]
	#[test_case(langid!("zh-Hant"); "stript")]
	#[test_case(langid!("ja-Jpan-JP"); "complex")]
	fn use_language(ln: LanguageIdentifier) {
		let file = ln.to_string().to_lowercase();
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_load_asset::<Ftl, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from(format!("my/path/{file}.ftl"))))
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
			mock.expect_load_asset::<Ftl, PathBuf>()
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
