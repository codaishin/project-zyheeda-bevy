use crate::resources::agents::color_lookup::AgentsLookupImages;
use bevy::prelude::*;
use common::traits::load_asset::LoadAsset;
use std::path::PathBuf;

const ROOT_PATH: &str = "maps/lookup";
const PLAYER_FILE: &str = "player.png";
const ENEMY_FILE: &str = "enemy.png";

impl AgentsLookupImages {
	pub(crate) fn lookup_images(commands: Commands, asset_server: ResMut<AssetServer>) {
		lookup_images(commands, asset_server)
	}
}

fn lookup_images<TAssetServer>(mut commands: Commands, mut asset_server: ResMut<TAssetServer>)
where
	TAssetServer: Resource + LoadAsset,
{
	let root = PathBuf::from(ROOT_PATH);
	let player = asset_server.load_asset(root.join(PLAYER_FILE));
	let enemy = asset_server.load_asset(root.join(ENEMY_FILE));

	commands.insert_resource(AgentsLookupImages::<Image> { player, enemy });
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::asset::AssetPath;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::path::PathBuf;
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Resource, NestedMocks)]
	struct _Assets {
		mock: Mock_Assets,
	}

	#[automock]
	impl LoadAsset for _Assets {
		fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_asset(path)
		}
	}

	fn setup(assets: _Assets) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(assets);
		app.add_systems(Update, lookup_images::<_Assets>);

		app
	}

	#[test]
	fn set_player() {
		let player = new_handle::<Image>();
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_load_asset::<Image, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from(ROOT_PATH).join(PLAYER_FILE)))
				.return_const(player.clone());
			mock.expect_load_asset::<Image, PathBuf>()
				.return_const(new_handle());
		}));

		app.update();

		assert_eq!(
			Some(&player),
			app.world()
				.get_resource::<AgentsLookupImages>()
				.map(|l| &l.player),
		);
	}

	#[test]
	fn set_enemy() {
		let enemy = new_handle::<Image>();
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_load_asset::<Image, PathBuf>()
				.times(1)
				.with(eq(PathBuf::from(ROOT_PATH).join(ENEMY_FILE)))
				.return_const(enemy.clone());
			mock.expect_load_asset::<Image, PathBuf>()
				.return_const(new_handle());
		}));

		app.update();

		assert_eq!(
			Some(&enemy),
			app.world()
				.get_resource::<AgentsLookupImages>()
				.map(|l| &l.enemy),
		);
	}
}
