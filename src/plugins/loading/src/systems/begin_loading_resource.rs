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
		let handle: Handle<TAsset> = server.load_asset(&path);
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
	use common::traits::load_asset::mock::MockAssetServer;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Asset;

	fn setup(path: Path, server: MockAssetServer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(server);
		app.add_systems(Update, begin_loading::<_Asset, MockAssetServer>(path));

		app
	}

	#[test]
	fn call_asset_server_load() {
		let server = MockAssetServer::default();
		let mut app = setup(Path::from("my/path"), server);

		app.update();

		assert_eq!(
			1,
			app.world().resource::<MockAssetServer>().calls("my/path"),
		)
	}

	#[test]
	fn store_asset_handle() {
		let handle = new_handle();
		let server = MockAssetServer::default()
			.path("my/path")
			.returns(handle.clone());
		let mut app = setup(Path::from("my/path"), server);

		app.update();

		assert_eq!(
			Some(&AssetResourceHandle(handle)),
			app.world().get_resource::<AssetResourceHandle<_Asset>>()
		);
	}
}
