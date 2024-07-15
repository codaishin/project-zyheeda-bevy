use crate::traits::Orbit;
use bevy::{input::mouse::MouseMotion, prelude::*};

pub(crate) fn move_on_orbit<TOrbitComponent: Orbit + Component>(
	mouse: Res<ButtonInput<MouseButton>>,
	mut mouse_motion: EventReader<MouseMotion>,
	mut query: Query<(&TOrbitComponent, &mut Transform)>,
) {
	if !mouse.pressed(MouseButton::Right) {
		return;
	}

	let Ok((orbit, mut transform)) = query.get_single_mut() else {
		return;
	};

	for event in mouse_motion.read() {
		orbit.orbit(&mut transform, event.delta);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::{Orbit, Vec2Radians};
	use bevy::{ecs::event::Events, input::mouse::MouseMotion};
	use common::test_tools::utils::SingleThreadedApp;
	use mockall::{automock, predicate::eq};

	#[derive(Component)]
	struct _Orbit {
		mock: Mock_Orbit,
	}

	impl _Orbit {
		pub fn new() -> Self {
			Self {
				mock: Mock_Orbit::new(),
			}
		}
	}

	#[automock]
	impl Orbit for _Orbit {
		fn orbit(&self, agent: &mut Transform, angles: Vec2Radians) {
			self.mock.orbit(agent, angles);
		}
	}

	fn setup_app() -> App {
		let mut app = App::new().single_threaded(Update);
		let input = ButtonInput::<MouseButton>::default();

		app.add_systems(Update, move_on_orbit::<_Orbit>);
		app.add_event::<MouseMotion>();
		app.insert_resource(input);

		app
	}

	#[test]
	fn move_camera_on_move_event() {
		let mut app = setup_app();
		let mut orbit = _Orbit::new();
		let agent = Transform::from_translation(Vec3::ZERO);
		let angels = Vec2::new(3., 4.);

		orbit
			.mock
			.expect_orbit()
			.with(eq(agent), eq(angels))
			.times(1)
			.return_const(());

		app.world_mut().spawn((orbit, agent));
		app.world_mut()
			.resource_mut::<Events<MouseMotion>>()
			.send(MouseMotion { delta: angels });
		app.world_mut()
			.resource_mut::<ButtonInput<MouseButton>>()
			.press(MouseButton::Right);

		app.update();
	}

	#[test]
	fn do_not_move_camera_when_not_right_mouse_button_pressed() {
		let mut app = setup_app();
		let mut orbit = _Orbit::new();
		let agent = Transform::from_translation(Vec3::ZERO);
		let angels = Vec2::new(3., 4.);

		orbit
			.mock
			.expect_orbit()
			.with(eq(agent), eq(angels))
			.times(0)
			.return_const(());

		app.world_mut().spawn((orbit, agent));
		app.world_mut()
			.resource_mut::<Events<MouseMotion>>()
			.send(MouseMotion { delta: angels });

		app.update();
	}
}
