use crate::traits::orbit::Orbit;
use bevy::{input::mouse::MouseMotion, prelude::*};
use common::tools::keys::user_input::UserInput;

pub(crate) fn move_on_orbit<TOrbit>(
	mouse: Res<ButtonInput<UserInput>>,
	mut mouse_motion: EventReader<MouseMotion>,
	mut query: Query<(&TOrbit, &mut Transform)>,
) where
	TOrbit: Orbit + Component,
{
	if !mouse.pressed(UserInput::from(MouseButton::Right)) {
		return;
	}

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
	use bevy::{ecs::event::Events, input::mouse::MouseMotion};
	use common::{test_tools::utils::SingleThreadedApp, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

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

	fn setup_app() -> App {
		let mut app = App::new().single_threaded(Update);
		let input = ButtonInput::<UserInput>::default();

		app.add_systems(Update, move_on_orbit::<_Orbit>);
		app.add_event::<MouseMotion>();
		app.insert_resource(input);

		app
	}

	#[test]
	fn move_camera_on_move_event() {
		let mut app = setup_app();
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
			.resource_mut::<Events<MouseMotion>>()
			.send(MouseMotion {
				delta: Vec2::new(3., 4.),
			});
		app.world_mut()
			.resource_mut::<ButtonInput<UserInput>>()
			.press(UserInput::from(MouseButton::Right));

		app.update();
	}

	#[test]
	fn do_not_move_camera_when_not_right_mouse_button_pressed() {
		let mut app = setup_app();
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
			.resource_mut::<Events<MouseMotion>>()
			.send(MouseMotion {
				delta: Vec2::new(3., 4.),
			});

		app.update();
	}

	#[test]
	fn move_multiple_cameras_on_move_event() {
		let mut app = setup_app();
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
			.resource_mut::<Events<MouseMotion>>()
			.send(MouseMotion {
				delta: Vec2::new(3., 4.),
			});
		app.world_mut()
			.resource_mut::<ButtonInput<UserInput>>()
			.press(UserInput::from(MouseButton::Right));

		app.update();
	}
}
