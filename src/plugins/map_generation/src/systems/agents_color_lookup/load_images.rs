use crate::resources::agents::color_lookup::AgentsColorLookupImages;
use bevy::prelude::*;
use common::traits::{handles_enemies::EnemyType, iteration::IterFinite, load_asset::LoadAsset};
use std::{collections::HashMap, path::PathBuf};

impl AgentsColorLookupImages {
	pub(crate) fn lookup_images(commands: Commands, asset_server: ResMut<AssetServer>) {
		lookup_images(commands, asset_server)
	}
}

fn lookup_images<TAssetServer>(mut commands: Commands, mut asset_server: ResMut<TAssetServer>)
where
	TAssetServer: Resource + LoadAsset,
{
	let root = PathBuf::from(AgentsColorLookupImages::ROOT_PATH);
	let player = asset_server.load_asset(root.join(AgentsColorLookupImages::PLAYER_FILE));
	let enemies = HashMap::from_iter(
		EnemyType::iterator()
			.map(|e| (e, AgentsColorLookupImages::get_enemy_file(&e)))
			.map(|(enemy_type, file)| (enemy_type, asset_server.load_asset(root.join(file)))),
	);

	commands.insert_resource(AgentsColorLookupImages::<Image> { player, enemies });
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::{
		handles_enemies::EnemyType,
		iteration::IterFinite,
		load_asset::mock::MockAssetServer,
	};
	use testing::{SingleThreadedApp, new_handle};

	fn setup(assets: MockAssetServer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(assets);
		app.add_systems(Update, lookup_images::<MockAssetServer>);

		app
	}

	#[test]
	fn set_player() {
		let player = new_handle::<Image>();
		let mut app = setup(
			MockAssetServer::default()
				.path(format!(
					"{}/{}",
					AgentsColorLookupImages::ROOT_PATH,
					AgentsColorLookupImages::PLAYER_FILE
				))
				.returns(player.clone()),
		);

		app.update();

		assert_eq!(
			Some(&player),
			app.world()
				.get_resource::<AgentsColorLookupImages>()
				.map(|l| &l.player),
		);
	}

	#[test]
	fn set_enemy() {
		let mut enemy_handles = HashMap::default();
		let mut server = MockAssetServer::default();
		for enemy_type in EnemyType::iterator() {
			let file = AgentsColorLookupImages::get_enemy_file(&enemy_type);
			let enemy = new_handle::<Image>();
			enemy_handles.insert(enemy_type, enemy.clone());
			server = server
				.path(format!("{}/{}", AgentsColorLookupImages::ROOT_PATH, file))
				.returns(enemy);
		}
		let mut app = setup(server);

		app.update();

		assert_eq!(
			Some(&enemy_handles),
			app.world()
				.get_resource::<AgentsColorLookupImages>()
				.map(|l| &l.enemies),
		);
	}
}
