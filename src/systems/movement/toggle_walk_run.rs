use crate::{behavior::MovementMode, components::Player};
use bevy::prelude::*;

pub fn toggle_walk_run(mut player: Query<&mut Player>, keys: Res<Input<KeyCode>>) {
	let Ok(mut player) = player.get_single_mut() else {
		return; //FIXME: handle properly
	};

	if !keys.just_pressed(KeyCode::NumpadSubtract) {
		return;
	}

	player.movement_mode = match player.movement_mode {
		MovementMode::Run => MovementMode::Walk,
		MovementMode::Walk => MovementMode::Run,
	};
}

#[cfg(test)]
mod tests {
	use crate::{
		behavior::MovementMode,
		components::{Player, UnitsPerSecond},
	};

	use super::*;

	#[test]
	fn toggle_player_walk_to_run() {
		let mut app = App::new();
		let keys = Input::<KeyCode>::default();
		let player = Player {
			movement_speed: UnitsPerSecond::new(0.1),
			run_speed: UnitsPerSecond::new(0.2),
			movement_mode: MovementMode::Walk,
		};

		let player = app.world.spawn(player).id();
		app.add_systems(Update, toggle_walk_run);
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
			movement_speed: UnitsPerSecond::new(0.1),
			run_speed: UnitsPerSecond::new(0.2),
			movement_mode: MovementMode::Run,
		};

		let player = app.world.spawn(player).id();
		app.add_systems(Update, toggle_walk_run);
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
			movement_speed: UnitsPerSecond::new(0.1),
			run_speed: UnitsPerSecond::new(0.2),
			movement_mode: MovementMode::Walk,
		};

		let player = app.world.spawn(player).id();
		app.add_systems(Update, toggle_walk_run);
		app.insert_resource(keys);

		app.update();

		let player = app.world.entity(player).get::<Player>().unwrap();
		assert_eq!(MovementMode::Walk, player.movement_mode);
	}
}
