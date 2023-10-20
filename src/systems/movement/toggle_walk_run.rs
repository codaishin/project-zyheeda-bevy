use crate::{
	behavior::{Run, Walk},
	components::Player,
};
use bevy::prelude::*;

type PlayerData<'a> = (Entity, Option<&'a Walk>, Option<&'a Run>);

pub fn player_toggle_walk_run(
	mut commands: Commands,
	player: Query<PlayerData, With<Player>>,
	keys: Res<Input<KeyCode>>,
) {
	if !keys.just_pressed(KeyCode::NumpadSubtract) {
		return;
	}

	for (player, walk, run) in player.iter() {
		let mut player = commands.entity(player);
		match (walk, run) {
			(Some(_), None) => {
				player.insert(Run);
				player.remove::<Walk>();
			}
			_ => {
				player.insert(Walk);
				player.remove::<Run>();
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Player;

	#[test]
	fn toggle_player_walk_to_run() {
		let mut app = App::new();
		let keys = Input::<KeyCode>::default();
		let player = Player { ..default() };

		let player = app.world.spawn((player, Walk)).id();
		app.add_systems(Update, player_toggle_walk_run);
		app.insert_resource(keys);
		app.world
			.resource_mut::<Input<KeyCode>>()
			.press(KeyCode::NumpadSubtract);

		app.update();

		let player = app.world.entity(player);
		assert_eq!(
			(false, true),
			(player.contains::<Walk>(), player.contains::<Run>())
		);
	}

	#[test]
	fn toggle_player_run_to_walk() {
		let mut app = App::new();
		let keys = Input::<KeyCode>::default();
		let player = Player { ..default() };

		let player = app.world.spawn((player, Run)).id();
		app.add_systems(Update, player_toggle_walk_run);
		app.insert_resource(keys);
		app.world
			.resource_mut::<Input<KeyCode>>()
			.press(KeyCode::NumpadSubtract);

		app.update();

		let player = app.world.entity(player);
		assert_eq!(
			(true, false),
			(player.contains::<Walk>(), player.contains::<Run>())
		);
	}

	#[test]
	fn no_toggle_when_no_input() {
		let mut app = App::new();
		let keys = Input::<KeyCode>::default();
		let player = Player { ..default() };

		let player = app.world.spawn((player, Walk)).id();
		app.add_systems(Update, player_toggle_walk_run);
		app.insert_resource(keys);

		app.update();

		let player = app.world.entity(player);
		assert_eq!(
			(true, false),
			(player.contains::<Walk>(), player.contains::<Run>())
		);
	}
}
