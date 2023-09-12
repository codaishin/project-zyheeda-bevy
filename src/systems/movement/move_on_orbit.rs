use crate::traits::orbit::Orbit;
use bevy::{
	input::{mouse::MouseMotion, *},
	prelude::*,
};

pub fn move_on_orbit<TOrbitComponent: Orbit + Component>(
	mouse: Res<Input<MouseButton>>,
	mut mouse_motion: EventReader<MouseMotion>,
	mut query: Query<(&TOrbitComponent, &mut Transform)>,
) {
	if !mouse.pressed(MouseButton::Right) {
		return;
	}
	for event in mouse_motion.iter() {
		for (orbit, mut transform) in query.iter_mut() {
			orbit.orbit(&mut transform, event.delta);
		}
	}
}

#[cfg(test)]
mod tests {
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
		let mut input = Input::<MouseButton>::default();
		let mut app = App::new();

		orbit
			.mock
			.expect_orbit()
			.with(eq(agent), eq(angels))
			.times(1)
			.return_const(());

		app.add_systems(Update, move_on_orbit::<_Orbit>);
		app.add_event::<MouseMotion>();
		app.world.spawn((orbit, agent));
		app.world
			.resource_mut::<Events<MouseMotion>>()
			.send(MouseMotion { delta: angels });
		input.press(MouseButton::Right);
		app.insert_resource(input);

		app.update();
	}

	#[test]
	fn do_not_move_camera_when_not_right_mouse_button_pressed() {
		let mut orbit = _Orbit::new();
		let agent = Transform::from_translation(Vec3::ZERO);
		let angels = Vec2::new(3., 4.);
		let input = Input::<MouseButton>::default();
		let mut app = App::new();

		orbit
			.mock
			.expect_orbit()
			.with(eq(agent), eq(angels))
			.times(0)
			.return_const(());

		app.add_systems(Update, move_on_orbit::<_Orbit>);
		app.add_event::<MouseMotion>();
		app.world.spawn((orbit, agent));
		app.world
			.resource_mut::<Events<MouseMotion>>()
			.send(MouseMotion { delta: angels });
		app.insert_resource(input);

		app.update();
	}
}
