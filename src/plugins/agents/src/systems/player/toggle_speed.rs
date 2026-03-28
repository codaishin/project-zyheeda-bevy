use crate::components::player::Player;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::action_key::movement::MovementKey,
	traits::{
		accessors::get::GetContextMut,
		handles_input::{GetAllInputStates, InputState},
		handles_movement::{ConfiguredMovement, ToggleSpeed},
	},
};

impl Player {
	pub(crate) fn toggle_speed<TInput, TMovement>(
		input: StaticSystemParam<TInput>,
		mut movement: StaticSystemParam<TMovement>,
		players: Query<Entity, With<Self>>,
	) where
		for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetAllInputStates>,
		for<'c> TMovement:
			SystemParam + GetContextMut<ConfiguredMovement, TContext<'c>: ToggleSpeed>,
	{
		let just_toggled = input
			.get_all_input_states::<MovementKey>()
			.any(just_toggled);

		if !just_toggled {
			return;
		}

		for entity in players {
			let key = ConfiguredMovement { entity };
			let Some(mut ctx) = TMovement::get_context_mut(&mut movement, key) else {
				continue;
			};

			ctx.toggle_speed();
		}
	}
}

fn just_toggled((key, state): (MovementKey, InputState)) -> bool {
	key == MovementKey::ToggleWalkRun && state == InputState::just_pressed()
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::action_key::{ActionKey, movement::MovementKey},
		traits::{handles_input::InputState, handles_movement::SpeedToggle, iteration::IterFinite},
	};
	use macros::NestedMocks;
	use mockall::automock;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Input {
		mock: Mock_Input,
	}

	#[automock]
	impl GetAllInputStates for _Input {
		fn get_all_input_states<TAction>(&self) -> impl Iterator<Item = (TAction, InputState)>
		where
			TAction: Into<ActionKey> + IterFinite + 'static,
		{
			self.mock.get_all_input_states()
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Movement {
		mock: Mock_Movement,
	}

	#[automock]
	impl ToggleSpeed for _Movement {
		fn toggle_speed(&mut self) -> SpeedToggle {
			self.mock.toggle_speed()
		}
	}

	fn setup(input: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			Player::toggle_speed::<Res<_Input>, Query<&mut _Movement>>,
		);
		app.insert_resource(input);

		app
	}

	mod with_input {
		use super::*;

		#[test]
		fn toggle() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::just_pressed(),
						)))
					});
			}));
			app.world_mut()
				.spawn((Player, _Movement::new().with_mock(assert_toggle_once)));

			app.update();

			fn assert_toggle_once(mock: &mut Mock_Movement) {
				mock.expect_toggle_speed()
					.times(1)
					.return_const(SpeedToggle::default());
			}
		}
	}

	mod no_toggle_input {
		use super::*;

		#[test]
		fn do_not_toggle() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::pressed(),
						)))
					});
			}));
			app.world_mut()
				.spawn((Player, _Movement::new().with_mock(assert_toggle_never)));

			app.update();

			fn assert_toggle_never(mock: &mut Mock_Movement) {
				mock.expect_toggle_speed()
					.never()
					.return_const(SpeedToggle::default());
			}
		}
	}

	mod no_player {
		use super::*;

		#[test]
		fn do_not_toggle() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::just_pressed(),
						)))
					});
			}));
			app.world_mut()
				.spawn(_Movement::new().with_mock(assert_toggle_never));

			app.update();

			fn assert_toggle_never(mock: &mut Mock_Movement) {
				mock.expect_toggle_speed()
					.never()
					.return_const(SpeedToggle::default());
			}
		}
	}
}
