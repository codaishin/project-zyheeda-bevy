use behaviors::components::{MovementConfig, MovementMode};
use bevy::prelude::*;
use common::components::Player;

pub fn player_toggle_walk_run(
	mut player: Query<&mut MovementConfig, With<Player>>,
	keys: Res<Input<KeyCode>>,
) {
	if !keys.just_pressed(KeyCode::NumpadSubtract) {
		return;
	}

	for mut config in player.iter_mut() {
		update_config(&mut config);
	}
}

fn update_config(config: &mut MovementConfig) {
	let MovementConfig::Dynamic { current_mode, .. } = config else {
		return;
	};
	*current_mode = match current_mode {
		MovementMode::Slow => MovementMode::Fast,
		MovementMode::Fast => MovementMode::Slow,
	};
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn toggle_player_walk_to_run() {
		let mut app = App::new();
		let keys = Input::<KeyCode>::default();
		let config = MovementConfig::Dynamic {
			current_mode: MovementMode::Slow,
			fast_speed: default(),
			slow_speed: default(),
		};

		let player = app.world.spawn((Player, config)).id();
		app.add_systems(Update, player_toggle_walk_run);
		app.insert_resource(keys);
		app.world
			.resource_mut::<Input<KeyCode>>()
			.press(KeyCode::NumpadSubtract);

		app.update();

		let current = app
			.world
			.entity(player)
			.get::<MovementConfig>()
			.and_then(|m| match m {
				MovementConfig::Dynamic { current_mode, .. } => Some(current_mode),
				_ => None,
			});
		assert_eq!(Some(&MovementMode::Fast), current);
	}

	#[test]
	fn toggle_player_run_to_walk() {
		let mut app = App::new();
		let keys = Input::<KeyCode>::default();
		let config = MovementConfig::Dynamic {
			current_mode: MovementMode::Fast,
			fast_speed: default(),
			slow_speed: default(),
		};

		let player = app.world.spawn((Player, config)).id();
		app.add_systems(Update, player_toggle_walk_run);
		app.insert_resource(keys);
		app.world
			.resource_mut::<Input<KeyCode>>()
			.press(KeyCode::NumpadSubtract);

		app.update();

		let current = app
			.world
			.entity(player)
			.get::<MovementConfig>()
			.and_then(|m| match m {
				MovementConfig::Dynamic { current_mode, .. } => Some(current_mode),
				_ => None,
			});
		assert_eq!(Some(&MovementMode::Slow), current);
	}

	#[test]
	fn no_toggle_when_no_input() {
		let mut app = App::new();
		let keys = Input::<KeyCode>::default();
		let config = MovementConfig::Dynamic {
			current_mode: MovementMode::Slow,
			fast_speed: default(),
			slow_speed: default(),
		};

		let player = app.world.spawn((Player, config)).id();
		app.add_systems(Update, player_toggle_walk_run);
		app.insert_resource(keys);

		app.update();

		let current = app
			.world
			.entity(player)
			.get::<MovementConfig>()
			.and_then(|m| match m {
				MovementConfig::Dynamic { current_mode, .. } => Some(current_mode),
				_ => None,
			});
		assert_eq!(Some(&MovementMode::Slow), current);
	}
}
