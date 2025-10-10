use crate::components::{
	player::Player,
	player_movement::{MovementMode, PlayerMovement},
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::action_key::movement::MovementKey,
	traits::handles_input::{GetInputState, InputState},
};

pub fn player_toggle_walk_run<TInput>(
	mut player: Query<&mut PlayerMovement, With<Player>>,
	input: StaticSystemParam<TInput>,
) where
	for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetInputState>,
{
	if input.get_input_state(MovementKey::ToggleWalkRun) != InputState::just_pressed() {
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
	use common::tools::action_key::{ActionKey, user_input::UserInput};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Input {
		mock: Mock_Input,
	}

	#[automock]
	impl GetInputState for _Input {
		fn get_input_state<TAction>(&self, action: TAction) -> InputState
		where
			TAction: Into<ActionKey> + 'static,
		{
			self.mock.get_input_state(action)
		}
	}

	fn setup(map: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(map);
		app.init_resource::<ButtonInput<UserInput>>();
		app.add_systems(Update, player_toggle_walk_run::<Res<_Input>>);

		app
	}

	#[test]
	fn toggle_player_walk_to_run() {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input_state()
				.with(eq(MovementKey::ToggleWalkRun))
				.return_const(InputState::just_pressed());
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
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input_state()
				.with(eq(MovementKey::ToggleWalkRun))
				.return_const(InputState::just_pressed());
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
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input_state()
				.with(eq(MovementKey::ToggleWalkRun))
				.return_const(InputState::released());
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
}
