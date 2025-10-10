use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::action_key::movement::MovementKey,
	traits::{
		handles_input::{GetInputState, InputState},
		intersect_at::IntersectAt,
	},
};

use crate::systems::movement::insert_process_component::ProcessInput;

impl<T> ParsePointerMovement for T where T: PointMovementInput {}

pub(crate) trait ParsePointerMovement: PointMovementInput {
	fn parse<TRay, TInput>(
		input: StaticSystemParam<TInput>,
		cam_ray: Res<TRay>,
	) -> ProcessInput<Self>
	where
		TRay: IntersectAt + Resource,
		for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetInputState>,
	{
		let InputState::Pressed { .. } = input.get_input_state(MovementKey::Pointer) else {
			return ProcessInput::None;
		};

		let Some(intersection) = cam_ray.intersect_at(0.) else {
			return ProcessInput::None;
		};

		ProcessInput::New(Self::from(intersection))
	}
}

pub(crate) trait PointMovementInput: From<Vec3> {}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
		math::Vec3,
	};
	use common::{
		tools::action_key::{ActionKey, user_input::UserInput},
		traits::{handles_input::GetInputState, intersect_at::IntersectAt},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use test_case::test_case;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _Result(Vec3);

	impl From<Vec3> for _Result {
		fn from(translation: Vec3) -> Self {
			Self(translation)
		}
	}

	impl PointMovementInput for _Result {}

	#[derive(Resource, NestedMocks)]
	struct _Ray {
		mock: Mock_Ray,
	}

	#[automock]
	impl IntersectAt for _Ray {
		fn intersect_at(&self, height: f32) -> Option<Vec3> {
			self.mock.intersect_at(height)
		}
	}

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

	fn setup(ray: _Ray, input: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(ray);
		app.insert_resource(input);
		app.init_resource::<ButtonInput<UserInput>>();

		app
	}

	#[test_case(InputState::pressed(); "on press")]
	#[test_case(InputState::just_pressed(); "on just press")]
	fn trigger(state: InputState) -> Result<(), RunSystemError> {
		let mut app = setup(
			_Ray::new().with_mock(|mock| {
				mock.expect_intersect_at()
					.return_const(Vec3::new(1., 2., 3.));
			}),
			_Input::new().with_mock(|mock| {
				mock.expect_get_input_state()
					.with(eq(MovementKey::Pointer))
					.return_const(state);
			}),
		);

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<_Ray, Res<_Input>>)?;

		assert_eq!(ProcessInput::New(_Result(Vec3::new(1., 2., 3.))), input);
		Ok(())
	}

	#[test_case(InputState::released(); "released")]
	#[test_case(InputState::just_released(); "just released")]
	fn no_trigger_when_not_pressed(state: InputState) -> Result<(), RunSystemError> {
		let mut app = setup(
			_Ray::new().with_mock(|mock| {
				mock.expect_intersect_at().return_const(Vec3::default());
			}),
			_Input::new().with_mock(|mock| {
				mock.expect_get_input_state()
					.with(eq(MovementKey::Pointer))
					.return_const(state);
			}),
		);

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<_Ray, Res<_Input>>)?;

		assert_eq!(ProcessInput::None, input);
		Ok(())
	}

	#[test]
	fn no_event_when_no_intersection() -> Result<(), RunSystemError> {
		let mut app = setup(
			_Ray::new().with_mock(|mock| {
				mock.expect_intersect_at().return_const(None);
			}),
			_Input::new().with_mock(|mock| {
				mock.expect_get_input_state()
					.with(eq(MovementKey::Pointer))
					.return_const(InputState::just_pressed());
			}),
		);

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<_Ray, Res<_Input>>)?;

		assert_eq!(ProcessInput::None, input);
		Ok(())
	}

	#[test]
	fn call_intersect_with_height_zero() -> Result<(), RunSystemError> {
		let mut app = setup(
			_Ray::new().with_mock(|mock| {
				mock.expect_intersect_at()
					.with(eq(0.))
					.times(1)
					.return_const(None);
			}),
			_Input::new().with_mock(|mock| {
				mock.expect_get_input_state()
					.with(eq(MovementKey::Pointer))
					.return_const(InputState::just_pressed());
			}),
		);

		_ = app
			.world_mut()
			.run_system_once(_Result::parse::<_Ray, Res<_Input>>)?;
		Ok(())
	}
}
