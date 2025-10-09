use crate::components::{
	player::Player,
	player_movement::{MovementMode, PlayerMovement},
};
use bevy::prelude::*;
use common::{
	tools::action_key::{movement::MovementKey, user_input::UserInput},
	traits::key_mappings::JustPressed,
};

pub fn player_toggle_walk_run<TMap>(
	mut player: Query<&mut PlayerMovement, With<Player>>,
	map: Res<TMap>,
	input: Res<ButtonInput<UserInput>>,
) where
	TMap: JustPressed<MovementKey> + Resource,
{
	if !map.just_pressed(&input).any(is_toggle_walk_run) {
		return;
	}

	for mut movement in player.iter_mut() {
		toggle_movement(&mut movement);
	}
}

fn is_toggle_walk_run(key: MovementKey) -> bool {
	key == MovementKey::ToggleWalkRun
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
	use common::{tools::action_key::user_input::UserInput, traits::iteration::IterFinite};
	use macros::NestedMocks;
	use mockall::automock;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl JustPressed<MovementKey> for _Map {
		fn just_pressed(
			&self,
			input: &ButtonInput<UserInput>,
		) -> impl Iterator<Item = MovementKey> {
			self.mock.just_pressed(input)
		}
	}

	fn setup(map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(map);
		app.init_resource::<ButtonInput<UserInput>>();
		app.add_systems(Update, player_toggle_walk_run::<_Map>);

		app
	}

	#[test]
	fn toggle_player_walk_to_run() {
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_just_pressed()
				.returning(|_| Box::new(std::iter::once(MovementKey::ToggleWalkRun)));
		}));
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
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_just_pressed()
				.returning(|_| Box::new(std::iter::once(MovementKey::ToggleWalkRun)));
		}));
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
	fn no_toggle_when_no_toggle_input() {
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_just_pressed().returning(|_| {
				Box::new(MovementKey::iterator().filter(|key| key != &MovementKey::ToggleWalkRun))
			});
		}));
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

	#[test]
	fn pass_correct_input_to_map() {
		let mut input = ButtonInput::default();
		input.press(UserInput::from(KeyCode::Digit7));
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_just_pressed().returning(|input| {
				assert_eq!(
					vec![&UserInput::from(KeyCode::Digit7)],
					input.get_pressed().collect::<Vec<_>>()
				);
				Box::new(std::iter::empty())
			});
		}));
		app.insert_resource(input);
		app.world_mut().spawn((
			Player,
			PlayerMovement {
				mode: MovementMode::Slow,
				..default()
			},
		));

		app.update();
	}
}
