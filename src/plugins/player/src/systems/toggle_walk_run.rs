use crate::components::{
	player::Player,
	player_movement::{MovementMode, PlayerMovement},
};
use bevy::prelude::*;
use common::tools::action_key::user_input::UserInput;

pub fn player_toggle_walk_run(
	mut player: Query<&mut PlayerMovement, With<Player>>,
	keys: Res<ButtonInput<UserInput>>,
) {
	if !keys.just_pressed(UserInput::from(KeyCode::NumpadSubtract)) {
		return;
	}

	for mut movement in player.iter_mut() {
		toggle_movement(&mut movement);
	}
}

fn toggle_movement(PlayerMovement { mode, .. }: &mut PlayerMovement) {
	*mode = match mode {
		MovementMode::Slow => MovementMode::Fast,
		MovementMode::Fast => MovementMode::Slow,
	};
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	fn setup(press: UserInput) -> App {
		let mut keys = ButtonInput::<UserInput>::default();
		let mut app = App::new().single_threaded(Update);

		keys.press(press);
		app.insert_resource(keys);
		app.add_systems(Update, player_toggle_walk_run);

		app
	}

	#[test]
	fn toggle_player_walk_to_run() {
		let mut app = setup(UserInput::from(KeyCode::NumpadSubtract));
		let player = app
			.world_mut()
			.spawn((
				Player,
				PlayerMovement {
					mode: MovementMode::Slow,
					..default()
				},
			))
			.id();

		app.update();

		assert_eq!(
			Some(&PlayerMovement {
				mode: MovementMode::Fast,
				..default()
			}),
			app.world().entity(player).get::<PlayerMovement>()
		);
	}

	#[test]
	fn toggle_player_run_to_walk() {
		let mut app = setup(UserInput::from(KeyCode::NumpadSubtract));
		let player = app
			.world_mut()
			.spawn((
				Player,
				PlayerMovement {
					mode: MovementMode::Fast,
					..default()
				},
			))
			.id();

		app.update();

		assert_eq!(
			Some(&PlayerMovement {
				mode: MovementMode::Slow,
				..default()
			}),
			app.world().entity(player).get::<PlayerMovement>()
		);
	}

	#[test]
	fn no_toggle_when_no_input() {
		let mut app = setup(UserInput::from(KeyCode::NumpadSubtract));
		let player = app
			.world_mut()
			.spawn(PlayerMovement {
				mode: MovementMode::Slow,
				..default()
			})
			.id();

		app.update();

		assert_eq!(
			Some(&PlayerMovement {
				mode: MovementMode::Slow,
				..default()
			}),
			app.world().entity(player).get::<PlayerMovement>()
		);
	}
}
