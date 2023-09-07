use bevy::{ecs::event::Events, input::mouse::MouseMotion, prelude::*};
use mockall::{automock, predicate::eq};

use super::*;
use crate::traits::orbit::{Orbit, Vec2Radians};

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

#[test]
fn move_camera_on_move_event() {
	let mut orbit = _Orbit::new();
	let agent = Transform::from_translation(Vec3::ZERO);
	let angels = Vec2::new(3., 4.);
	let mut app = App::new();

	orbit
		.mock
		.expect_orbit()
		.with(eq(agent), eq(angels))
		.times(1)
		.return_const(());

	app.add_systems(Update, orbit_transform_on_mouse_motion::<_Orbit>);
	app.add_event::<MouseMotion>();
	app.world.spawn((orbit, agent));
	app.world
		.resource_mut::<Events<MouseMotion>>()
		.send(MouseMotion { delta: angels });

	app.update();
}
