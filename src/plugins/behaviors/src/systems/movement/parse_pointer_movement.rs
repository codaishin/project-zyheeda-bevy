use crate::systems::movement::insert_process_component::ProcessInput;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::action_key::movement::MovementKey,
	traits::{
		handles_input::{GetInputState, InputState},
		handles_physics::{MouseGroundHover, MouseGroundPoint, Raycast},
	},
};

impl<T> ParsePointerMovement for T where T: PointMovementInput {}

pub(crate) trait ParsePointerMovement: PointMovementInput {
	fn parse<TInput, TRaycast>(
		input: StaticSystemParam<TInput>,
		mut cam_ray: StaticSystemParam<TRaycast>,
	) -> ProcessInput<Self>
	where
		TInput: for<'w, 's> SystemParam<Item<'w, 's>: GetInputState>,
		TRaycast: for<'w, 's> SystemParam<Item<'w, 's>: Raycast<MouseGroundHover>>,
	{
		let InputState::Pressed { .. } = input.get_input_state(MovementKey::Pointer) else {
			return ProcessInput::None;
		};

		let Some(MouseGroundPoint(intersection)) = cam_ray.raycast(MouseGroundHover) else {
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
	use common::{tools::action_key::ActionKey, traits::handles_input::GetInputState};
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
	struct _RayCaster {
		mock: Mock_RayCaster,
	}

	#[automock]
	impl Raycast<MouseGroundHover> for _RayCaster {
		fn raycast(&mut self, args: MouseGroundHover) -> Option<MouseGroundPoint> {
			self.mock.raycast(args)
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

	fn setup(ray: _RayCaster, input: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(ray);
		app.insert_resource(input);

		app
	}

	#[test_case(InputState::pressed(); "on press")]
	#[test_case(InputState::just_pressed(); "on just press")]
	fn trigger(state: InputState) -> Result<(), RunSystemError> {
		let mut app = setup(
			_RayCaster::new().with_mock(|mock| {
				mock.expect_raycast()
					.return_const(MouseGroundPoint(Vec3::new(1., 2., 3.)));
			}),
			_Input::new().with_mock(|mock| {
				mock.expect_get_input_state()
					.with(eq(MovementKey::Pointer))
					.return_const(state);
			}),
		);

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<Res<_Input>, ResMut<_RayCaster>>)?;

		assert_eq!(ProcessInput::New(_Result(Vec3::new(1., 2., 3.))), input);
		Ok(())
	}

	#[test_case(InputState::released(); "released")]
	#[test_case(InputState::just_released(); "just released")]
	fn no_trigger_when_not_pressed(state: InputState) -> Result<(), RunSystemError> {
		let mut app = setup(
			_RayCaster::new().with_mock(|mock| {
				mock.expect_raycast()
					.return_const(MouseGroundPoint::default());
			}),
			_Input::new().with_mock(|mock| {
				mock.expect_get_input_state()
					.with(eq(MovementKey::Pointer))
					.return_const(state);
			}),
		);

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<Res<_Input>, ResMut<_RayCaster>>)?;

		assert_eq!(ProcessInput::None, input);
		Ok(())
	}

	#[test]
	fn no_event_when_no_intersection() -> Result<(), RunSystemError> {
		let mut app = setup(
			_RayCaster::new().with_mock(|mock| {
				mock.expect_raycast().return_const(None);
			}),
			_Input::new().with_mock(|mock| {
				mock.expect_get_input_state()
					.with(eq(MovementKey::Pointer))
					.return_const(InputState::just_pressed());
			}),
		);

		let input = app
			.world_mut()
			.run_system_once(_Result::parse::<Res<_Input>, ResMut<_RayCaster>>)?;

		assert_eq!(ProcessInput::None, input);
		Ok(())
	}
}
