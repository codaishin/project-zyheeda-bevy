use crate::traits::orbit::Orbit;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	input::mouse::MouseMotion,
	prelude::*,
};
use common::{
	tools::action_key::camera_key::CameraKey,
	traits::handles_input::{GetInputState, InputState},
};

pub(crate) fn move_on_orbit<TOrbit, TInput>(
	input: StaticSystemParam<TInput>,
	mut mouse_motion: MessageReader<MouseMotion>,
	mut query: Query<(&TOrbit, &mut Transform)>,
) where
	TOrbit: Orbit + Component,
	for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetInputState>,
{
	let InputState::Pressed { .. } = input.get_input_state(CameraKey::Rotate) else {
		return;
	};

	for event in mouse_motion.read() {
		for (orbit, mut transform) in &mut query {
			orbit.orbit(&mut transform, event.delta);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::orbit::Vec2Radians;
	use bevy::input::mouse::MouseMotion;
	use common::tools::action_key::ActionKey;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use test_case::test_case;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component, NestedMocks)]
	struct _Orbit {
		mock: Mock_Orbit,
	}

	#[automock]
	impl Orbit for _Orbit {
		fn orbit(&self, agent: &mut Transform, angles: Vec2Radians) {
			self.mock.orbit(agent, angles);
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

	fn setup_app(map: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(map);
		app.add_systems(Update, move_on_orbit::<_Orbit, Res<_Input>>);
		app.add_message::<MouseMotion>();

		app
	}

	#[test_case(InputState::pressed(); "pressed")]
	#[test_case(InputState::just_pressed(); "just pressed")]
	fn move_camera_on(state: InputState) {
		let mut app = setup_app(_Input::new().with_mock(|mock| {
			mock.expect_get_input_state()
				.with(eq(CameraKey::Rotate))
				.return_const(state);
		}));
		app.world_mut().spawn((
			_Orbit::new().with_mock(|mock| {
				mock.expect_orbit()
					.with(
						eq(Transform::from_translation(Vec3::ZERO)),
						eq(Vec2::new(3., 4.)),
					)
					.times(1)
					.return_const(());
			}),
			Transform::from_translation(Vec3::ZERO),
		));
		app.world_mut()
			.resource_mut::<Messages<MouseMotion>>()
			.write(MouseMotion {
				delta: Vec2::new(3., 4.),
			});

		app.update();
	}

	#[test_case(InputState::released(); "released")]
	#[test_case(InputState::just_released(); "just released")]
	fn do_not_move_camera_on(state: InputState) {
		let mut app = setup_app(_Input::new().with_mock(|mock| {
			mock.expect_get_input_state()
				.with(eq(CameraKey::Rotate))
				.return_const(state);
		}));
		app.world_mut().spawn((
			_Orbit::new().with_mock(|mock| {
				mock.expect_orbit()
					.with(
						eq(Transform::from_translation(Vec3::ZERO)),
						eq(Vec2::new(3., 4.)),
					)
					.times(0)
					.return_const(());
			}),
			Transform::from_translation(Vec3::ZERO),
		));
		app.world_mut()
			.resource_mut::<Messages<MouseMotion>>()
			.write(MouseMotion {
				delta: Vec2::new(3., 4.),
			});

		app.update();
	}

	#[test]
	fn move_multiple_cameras() {
		let mut app = setup_app(_Input::new().with_mock(|mock| {
			mock.expect_get_input_state()
				.with(eq(CameraKey::Rotate))
				.return_const(InputState::pressed());
		}));
		app.world_mut().spawn((
			_Orbit::new().with_mock(|mock| {
				mock.expect_orbit()
					.with(
						eq(Transform::from_translation(Vec3::ZERO)),
						eq(Vec2::new(3., 4.)),
					)
					.times(1)
					.return_const(());
			}),
			Transform::from_translation(Vec3::ZERO),
		));
		app.world_mut().spawn((
			_Orbit::new().with_mock(|mock| {
				mock.expect_orbit()
					.with(
						eq(Transform::from_translation(Vec3::ZERO)),
						eq(Vec2::new(3., 4.)),
					)
					.times(1)
					.return_const(());
			}),
			Transform::from_translation(Vec3::ZERO),
		));
		app.world_mut()
			.resource_mut::<Messages<MouseMotion>>()
			.write(MouseMotion {
				delta: Vec2::new(3., 4.),
			});

		app.update();
	}
}
