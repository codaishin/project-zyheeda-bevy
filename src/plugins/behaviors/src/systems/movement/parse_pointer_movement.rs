use bevy::prelude::*;
use common::{
	tools::action_key::{movement::MovementKey, user_input::UserInput},
	traits::{intersect_at::IntersectAt, key_mappings::Pressed},
};

impl<T> ParsePointerMovement for T where T: PointMovementInput {}

pub(crate) trait ParsePointerMovement: PointMovementInput {
	fn parse<TRay, TMap>(
		input: Res<ButtonInput<UserInput>>,
		map: Res<TMap>,
		cam_ray: Res<TRay>,
	) -> Option<Self>
	where
		TRay: IntersectAt + Resource,
		TMap: Pressed<MovementKey> + Resource,
	{
		if !map.pressed(&input).any(|key| key == MovementKey::Pointer) {
			return None;
		}
		let intersection = cam_ray.intersect_at(0.)?;
		Some(Self::from(intersection))
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
		test_tools::utils::SingleThreadedApp,
		traits::{intersect_at::IntersectAt, iteration::IterFinite, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _Input(Vec3);

	impl From<Vec3> for _Input {
		fn from(translation: Vec3) -> Self {
			Self(translation)
		}
	}

	impl PointMovementInput for _Input {}

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
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl Pressed<MovementKey> for _Map {
		fn pressed(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = MovementKey> {
			self.mock.pressed(input)
		}
	}

	fn setup(ray: _Ray, map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(ray);
		app.insert_resource(map);
		app.init_resource::<ButtonInput<UserInput>>();

		app
	}

	#[test]
	fn trigger_immediately_on_movement_pointer_press() -> Result<(), RunSystemError> {
		let mut app = setup(
			_Ray::new().with_mock(|mock| {
				mock.expect_intersect_at()
					.return_const(Vec3::new(1., 2., 3.));
			}),
			_Map::new().with_mock(|mock| {
				mock.expect_pressed()
					.returning(|_| Box::new(std::iter::once(MovementKey::Pointer)));
			}),
		);

		let input = app
			.world_mut()
			.run_system_once(_Input::parse::<_Ray, _Map>)?;

		assert_eq!(Some(_Input(Vec3::new(1., 2., 3.))), input);
		Ok(())
	}

	#[test]
	fn no_event_when_other_movement_button_pressed() -> Result<(), RunSystemError> {
		let mut app = setup(
			_Ray::new().with_mock(|mock| {
				mock.expect_intersect_at().return_const(Vec3::default());
			}),
			_Map::new().with_mock(|mock| {
				mock.expect_pressed().returning(|_| {
					Box::new(MovementKey::iterator().filter(|key| key != &MovementKey::Pointer))
				});
			}),
		);

		let input = app
			.world_mut()
			.run_system_once(_Input::parse::<_Ray, _Map>)?;

		assert_eq!(None, input);
		Ok(())
	}

	#[test]
	fn no_event_when_no_intersection() -> Result<(), RunSystemError> {
		let mut app = setup(
			_Ray::new().with_mock(|mock| {
				mock.expect_intersect_at().return_const(None);
			}),
			_Map::new().with_mock(|mock| {
				mock.expect_pressed()
					.returning(|_| Box::new(std::iter::once(MovementKey::Pointer)));
			}),
		);

		let input = app
			.world_mut()
			.run_system_once(_Input::parse::<_Ray, _Map>)?;

		assert_eq!(None, input);
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
			_Map::new().with_mock(|mock| {
				mock.expect_pressed()
					.returning(|_| Box::new(std::iter::once(MovementKey::Pointer)));
			}),
		);

		_ = app
			.world_mut()
			.run_system_once(_Input::parse::<_Ray, _Map>)?;
		Ok(())
	}

	#[test]
	fn call_map_with_correct_input() -> Result<(), RunSystemError> {
		let mut input = ButtonInput::default();
		input.press(UserInput::MouseButton(MouseButton::Back));
		let mut app = setup(
			_Ray::new().with_mock(|mock| {
				mock.expect_intersect_at().return_const(None);
			}),
			_Map::new().with_mock(|mock| {
				mock.expect_pressed().returning(|input| {
					assert_eq!(
						vec![&UserInput::MouseButton(MouseButton::Back)],
						input.get_pressed().collect::<Vec<_>>()
					);
					Box::new(std::iter::empty())
				});
			}),
		);
		app.insert_resource(input);

		_ = app
			.world_mut()
			.run_system_once(_Input::parse::<_Ray, _Map>)?;
		Ok(())
	}
}
