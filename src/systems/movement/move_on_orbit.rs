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

	let Ok((orbit, mut transform)) = query.get_single_mut() else {
		return;
	};

	for event in mouse_motion.iter() {
		orbit.orbit(&mut transform, event.delta);
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

	fn setup_app() -> App {
		let mut app = App::new();
		let input = Input::<MouseButton>::default();

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

		app.world.spawn((orbit, agent));
		app.world
			.resource_mut::<Events<MouseMotion>>()
			.send(MouseMotion { delta: angels });
		app.world
			.resource_mut::<Input<MouseButton>>()
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
}
