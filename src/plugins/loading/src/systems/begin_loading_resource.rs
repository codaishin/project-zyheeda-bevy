use bevy::prelude::*;
use common::traits::load_asset::{LoadAsset, Path};

impl<T> BeginLoadingResource for T where T: Asset {}

pub(crate) trait BeginLoadingResource: Asset + Sized {
	fn begin_loading(path: Path) -> impl Fn(Commands, ResMut<AssetServer>) {
		begin_loading::<Self, AssetServer>(path)
	}
}

fn begin_loading<TAsset, TServer>(path: Path) -> impl Fn(Commands, ResMut<TServer>)
where
	TAsset: Asset,
	TServer: LoadAsset + Resource,
{
	move |mut commands, mut server| {
		let handle = server.load_asset::<TAsset, Path>(path.clone());
		commands.insert_resource(AssetResourceHandle(handle));
	}
}

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct AssetResourceHandle<TAsset>(pub(crate) Handle<TAsset>)
where
	TAsset: Asset;

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::asset::AssetPath;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Asset;

	#[derive(Resource, NestedMocks)]
	struct _Server {
		mock: Mock_Server,
	}

	#[automock]
	impl LoadAsset for _Server {
		fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_asset(path)
		}
	}

	fn setup(path: Path, server: _Server) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(server);
		app.add_systems(Update, begin_loading::<_Asset, _Server>(path));

		app
	}

	#[test]
	fn call_asset_server_load() {
		let server = _Server::new().with_mock(|mock| {
			mock.expect_load_asset()
				.times(1)
				.with(eq(Path::from("my/path")))
				.return_const(new_handle::<_Asset>());
		});
		let mut app = setup(Path::from("my/path"), server);

		app.update();
	}

	#[test]
	fn store_asset_handle() {
		let handle = new_handle();
		let server = _Server::new().with_mock(|mock| {
			mock.expect_load_asset::<_Asset, Path>()
				.return_const(handle.clone());
		});
		let mut app = setup(Path::from("my/path"), server);

		app.update();

		assert_eq!(
			Some(&AssetResourceHandle(handle)),
			app.world().get_resource::<AssetResourceHandle<_Asset>>()
		);
	}
}
