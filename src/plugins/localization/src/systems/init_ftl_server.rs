use crate::assets::ftl::Ftl;
use bevy::prelude::*;
use common::traits::load_asset::{LoadAsset, Path};
use std::path::PathBuf;
use unic_langid::LanguageIdentifier;

impl<T> InitFtlServer for T where T: From<(LanguageIdentifier, Handle<Ftl>)> + Resource {}

pub(crate) trait InitFtlServer: From<(LanguageIdentifier, Handle<Ftl>)> + Resource {
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
	TFtlServer: From<(LanguageIdentifier, Handle<Ftl>)> + Resource,
	TAssetServer: LoadAsset + Resource,
{
	move |mut commands, mut server| {
		let ftl = ln.to_string().to_lowercase();
		let path = PathBuf::from(&*root_path).join(format!("{ftl}.ftl"));
		let handle = server.load_asset(path);
		let server = TFtlServer::from((ln.clone(), handle));
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
	use mockall::{automock, predicate::eq};
	use std::path::PathBuf;
	use unic_langid::langid;

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

	#[derive(Resource, Debug, PartialEq)]
	struct _FtlServer {
		ln: LanguageIdentifier,
		handle: Handle<Ftl>,
	}

	impl From<(LanguageIdentifier, Handle<Ftl>)> for _FtlServer {
		fn from((ln, handle): (LanguageIdentifier, Handle<Ftl>)) -> Self {
			Self { ln, handle }
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
		let handle = new_handle::<Ftl>();
		let handle_clone = handle.clone();
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_load_asset::<Ftl, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/jp.ftl")))
				.return_const(handle_clone.clone());
		});
		let mut app = setup(langid!("JP"), Path::from("my/path"), server);

		app.update();

		assert_eq!(
			Some(&_FtlServer {
				ln: langid!("JP"),
				handle
			}),
			app.world().get_resource::<_FtlServer>()
		);
	}

	#[test]
	fn init_with_language_region() {
		let handle = new_handle::<Ftl>();
		let handle_clone = handle.clone();
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_load_asset::<Ftl, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/en-us.ftl")))
				.return_const(handle_clone.clone());
		});
		let mut app = setup(langid!("en-US"), Path::from("my/path"), server);

		app.update();

		assert_eq!(
			Some(&_FtlServer {
				ln: langid!("en-US"),
				handle
			}),
			app.world().get_resource::<_FtlServer>()
		);
	}

	#[test]
	fn init_with_language_script() {
		let handle = new_handle::<Ftl>();
		let handle_clone = handle.clone();
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_load_asset::<Ftl, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/zh-hant.ftl")))
				.return_const(handle_clone.clone());
		});
		let mut app = setup(langid!("zh-Hant"), Path::from("my/path"), server);

		app.update();

		assert_eq!(
			Some(&_FtlServer {
				ln: langid!("zh-Hant"),
				handle
			}),
			app.world().get_resource::<_FtlServer>()
		);
	}

	#[test]
	fn init_with_language_complex() {
		let handle = new_handle::<Ftl>();
		let handle_clone = handle.clone();
		let server = _AssetServer::new().with_mock(move |mock| {
			mock.expect_load_asset::<Ftl, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from("my/path/ja-jpan-jp.ftl")))
				.return_const(handle_clone.clone());
		});
		let mut app = setup(langid!("ja-Jpan-JP"), Path::from("my/path"), server);

		app.update();

		assert_eq!(
			Some(&_FtlServer {
				ln: langid!("ja-Jpan-JP"),
				handle
			}),
			app.world().get_resource::<_FtlServer>()
		);
	}
}
