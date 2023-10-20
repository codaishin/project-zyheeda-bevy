use crate::{behavior::MovementMode, components::Player};
use bevy::prelude::*;

pub fn player_toggle_walk_run(mut player: Query<&mut Player>, keys: Res<Input<KeyCode>>) {
	if !keys.just_pressed(KeyCode::NumpadSubtract) {
		return;
	}

	for mut player in player.iter_mut() {
		player.movement_mode = match player.movement_mode {
			MovementMode::Walk => MovementMode::Run,
			MovementMode::Run => MovementMode::Walk,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{behavior::MovementMode, components::Player};

	#[test]
	fn toggle_player_walk_to_run() {
		let mut app = App::new();
		let keys = Input::<KeyCode>::default();
		let player = Player {
			movement_mode: MovementMode::Walk,
			..default()
		};

		let player = app.world.spawn(player).id();
		app.add_systems(Update, player_toggle_walk_run);
		app.insert_resource(keys);
		app.world
			.resource_mut::<Input<KeyCode>>()
			.press(KeyCode::NumpadSubtract);

		app.update();

		let player = app.world.entity(player).get::<Player>().unwrap();
		assert_eq!(MovementMode::Run, player.movement_mode);
	}

	#[test]
	fn toggle_player_run_to_walk() {
		let mut app = App::new();
		let keys = Input::<KeyCode>::default();
		let player = Player {
			movement_mode: MovementMode::Run,
			..default()
		};

		let player = app.world.spawn(player).id();
		app.add_systems(Update, player_toggle_walk_run);
		app.insert_resource(keys);
		app.world
			.resource_mut::<Input<KeyCode>>()
			.press(KeyCode::NumpadSubtract);

		app.update();

		let player = app.world.entity(player).get::<Player>().unwrap();
		assert_eq!(MovementMode::Walk, player.movement_mode);
	}

	#[test]
	fn no_toggle_when_no_input() {
		let mut app = App::new();
		let keys = Input::<KeyCode>::default();
		let player = Player {
			movement_mode: MovementMode::Walk,
			..default()
		};

		let player = app.world.spawn(player).id();
		app.add_systems(Update, player_toggle_walk_run);
		app.insert_resource(keys);

		app.update();

		let player = app.world.entity(player).get::<Player>().unwrap();
		assert_eq!(MovementMode::Walk, player.movement_mode);
	}
}
