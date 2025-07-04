use crate::traits::orbit::Orbit;
use bevy::{input::mouse::MouseMotion, prelude::*};
use common::{
	tools::action_key::{camera_key::CameraKey, user_input::UserInput},
	traits::key_mappings::Pressed,
};

pub(crate) fn move_on_orbit<TOrbit, TMap>(
	input: Res<ButtonInput<UserInput>>,
	map: Res<TMap>,
	mut mouse_motion: EventReader<MouseMotion>,
	mut query: Query<(&TOrbit, &mut Transform)>,
) where
	TOrbit: Orbit + Component,
	TMap: Pressed<CameraKey> + Resource,
{
	if !map.pressed(&input).any(|key| key == CameraKey::Rotate) {
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
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
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
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl Pressed<CameraKey> for _Map {
		fn pressed(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = CameraKey> {
			self.mock.pressed(input)
		}
	}

	fn setup_app(map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);
		let input = ButtonInput::<UserInput>::default();

		app.insert_resource(map);
		app.add_systems(Update, move_on_orbit::<_Orbit, _Map>);
		app.add_event::<MouseMotion>();
		app.insert_resource(input);

		app
	}

	#[test]
	fn move_camera_on_move_event() {
		let mut app = setup_app(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new(std::iter::once(CameraKey::Rotate)));
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
			.resource_mut::<Events<MouseMotion>>()
			.send(MouseMotion {
				delta: Vec2::new(3., 4.),
			});

		app.update();
	}

	#[test]
	fn do_not_move_camera_when_not_right_mouse_button_pressed() {
		let mut app = setup_app(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new(std::iter::empty()));
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
			.resource_mut::<Events<MouseMotion>>()
			.send(MouseMotion {
				delta: Vec2::new(3., 4.),
			});

		app.update();
	}

	#[test]
	fn move_multiple_cameras_on_move_event() {
		let mut app = setup_app(_Map::new().with_mock(|mock| {
			mock.expect_pressed()
				.returning(|_| Box::new(std::iter::once(CameraKey::Rotate)));
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
			.resource_mut::<Events<MouseMotion>>()
			.send(MouseMotion {
				delta: Vec2::new(3., 4.),
			});

		app.update();
	}

	#[test]
	fn call_map_with_correct_input() {
		let mut input = ButtonInput::default();
		input.press(UserInput::MouseButton(MouseButton::Right));
		let mut app = setup_app(_Map::new().with_mock(|mock| {
			mock.expect_pressed().returning(|input| {
				assert_eq!(
					vec![&UserInput::MouseButton(MouseButton::Right)],
					input.get_pressed().collect::<Vec<_>>()
				);
				Box::new(std::iter::once(CameraKey::Rotate))
			});
		}));
		app.insert_resource(input);
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

		app.update();
	}
}
