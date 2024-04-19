use crate::traits::Orbit;
use bevy::{input::mouse::MouseMotion, prelude::*};

pub(crate) fn move_on_orbit<TOrbitComponent: Orbit + Component>(
	mouse: Res<ButtonInput<MouseButton>>,
	mut mouse_motion: EventReader<MouseMotion>,
	mut query: Query<(&TOrbitComponent, &mut Transform)>,
) {
	if !mouse.pressed(MouseButton::Right) {
		// It seems odd to do this, but after moving this into a plugin we observe
		// events to persist more than 2 frames, so we clear manually.
		// This seems to be at odds with the documentation at https://docs.rs/bevy/0.12.1/bevy/prelude/struct.Events.html
		// but in line with comments in https://github.com/bevyengine/bevy/issues/10860.
		mouse_motion.clear();
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
		// Not using `add_event`, because we don't want events to be cleared between frames
		// so we can simulate the observed behavior of events not being cleared when running
		// the plugin within our game.
		app.init_resource::<Events<MouseMotion>>();
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

		app.world.spawn((orbit, agent));
		app.world
			.resource_mut::<Events<MouseMotion>>()
			.send(MouseMotion { delta: angels });
		app.world
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

		app.world.spawn((orbit, agent));
		app.world
			.resource_mut::<Events<MouseMotion>>()
			.send(MouseMotion { delta: angels });

		app.update();
	}

	#[test]
	fn disregard_events_that_happened_when_not_right_mouse_pressed() {
		let mut app = setup_app();
		let mut orbit = _Orbit::new();
		let agent = Transform::from_translation(Vec3::ZERO);
		let angels = Vec2::new(3., 4.);
		let discard_angles = Vec2::new(100., 200.);

		orbit
			.mock
			.expect_orbit()
			.with(eq(agent), eq(angels))
			.times(1)
			.return_const(());

		app.world.spawn((orbit, agent));

		app.update();

		app.world
			.resource_mut::<Events<MouseMotion>>()
			.send(MouseMotion {
				delta: discard_angles,
			});

		app.update();

		app.world
			.resource_mut::<Events<MouseMotion>>()
			.send(MouseMotion { delta: angels });
		app.world
			.resource_mut::<ButtonInput<MouseButton>>()
			.press(MouseButton::Right);

		app.update();
	}
}
