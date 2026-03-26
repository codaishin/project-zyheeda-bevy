use crate::components::{
	movement_config::{CurrentSpeed, MovementConfig, MovementSpeed},
	player::Player,
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::action_key::movement::MovementKey,
	traits::handles_input::{GetAllInputStates, InputState},
};

impl Player {
	pub(crate) fn toggle_speed<TInput>(
		input: StaticSystemParam<TInput>,
		players: Query<&mut MovementConfig, With<Self>>,
	) where
		for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetAllInputStates>,
	{
		let just_toggled = input
			.get_all_input_states::<MovementKey>()
			.any(just_toggled);

		if !just_toggled {
			return;
		}

		for mut config in players {
			let MovementSpeed::Variable(speed) = &mut config.speed else {
				continue;
			};

			speed.current = match &speed.current {
				CurrentSpeed::Walk => CurrentSpeed::Run,
				CurrentSpeed::Run => CurrentSpeed::Walk,
			};
		}
	}
}

fn just_toggled((key, state): (MovementKey, InputState)) -> bool {
	key == MovementKey::ToggleWalkRun && state == InputState::just_pressed()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::movement_config::VariableSpeed;
	use common::{
		tools::action_key::{ActionKey, movement::MovementKey},
		traits::{handles_input::InputState, iteration::IterFinite},
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

	fn setup(input: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Player::toggle_speed::<Res<_Input>>);
		app.insert_resource(input);

		app
	}

	mod run_to_walk {
		use super::*;

		#[test]
		fn set_walk() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::just_pressed(),
						)))
					});
			}));
			let entity = app
				.world_mut()
				.spawn((
					Player,
					MovementConfig::with_speed(VariableSpeed::from_current(CurrentSpeed::Run)),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&MovementSpeed::Variable(VariableSpeed::from_current(
					CurrentSpeed::Walk
				))),
				app.world()
					.entity(entity)
					.get::<MovementConfig>()
					.map(|c| &c.speed),
			);
		}
	}

	mod walk_to_run {
		use super::*;

		#[test]
		fn set_run() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::just_pressed(),
						)))
					});
			}));
			let entity = app
				.world_mut()
				.spawn((
					Player,
					MovementConfig::with_speed(VariableSpeed::from_current(CurrentSpeed::Walk)),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&MovementSpeed::Variable(VariableSpeed::from_current(
					CurrentSpeed::Run
				))),
				app.world()
					.entity(entity)
					.get::<MovementConfig>()
					.map(|c| &c.speed),
			);
		}
	}

	mod no_toggle_input {
		use super::*;

		#[test]
		fn do_not_call_update() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::pressed(),
						)))
					});
			}));
			app.world_mut().spawn((
				Player,
				MovementConfig::with_speed(VariableSpeed::from_current(CurrentSpeed::Run)),
			));

			app.update();
		}

		#[test]
		fn do_not_update_walk() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::pressed(),
						)))
					});
			}));
			let entity = app
				.world_mut()
				.spawn((
					Player,
					MovementConfig::with_speed(VariableSpeed::from_current(CurrentSpeed::Run)),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&MovementConfig::with_speed(VariableSpeed::from_current(
					CurrentSpeed::Run,
				))),
				app.world().entity(entity).get::<MovementConfig>(),
			);
		}
	}

	mod no_player {
		use super::*;

		#[test]
		fn do_not_call_update() {
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
				.spawn(MovementConfig::with_speed(VariableSpeed::from_current(
					CurrentSpeed::Run,
				)));

			app.update();
		}

		#[test]
		fn do_not_insert_config() {
			let mut app = setup(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states::<MovementKey>()
					.returning(|| {
						Box::new(std::iter::once((
							MovementKey::ToggleWalkRun,
							InputState::just_pressed(),
						)))
					});
			}));
			let entity = app
				.world_mut()
				.spawn(MovementConfig::with_speed(VariableSpeed::from_current(
					CurrentSpeed::Run,
				)))
				.id();

			app.update();

			assert_eq!(
				Some(&MovementConfig::with_speed(VariableSpeed::from_current(
					CurrentSpeed::Run,
				))),
				app.world().entity(entity).get::<MovementConfig>(),
			);
		}
	}
}
