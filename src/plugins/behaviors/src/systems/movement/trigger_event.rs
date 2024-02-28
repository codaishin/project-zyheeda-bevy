use crate::events::MoveInputEvent;
use bevy::{
	ecs::{event::EventWriter, system::Res},
	input::{mouse::MouseButton, Input},
	math::Vec3,
};
use common::resources::CamRay;

pub(crate) fn trigger_move_input_event(
	mouse_input: Res<Input<MouseButton>>,
	cam_ray: Res<CamRay>,
	mut move_input_events: EventWriter<MoveInputEvent>,
) {
	if !mouse_input.pressed(MouseButton::Left) {
		return;
	}
	let Some(ray) = cam_ray.0 else {
		return;
	};
	let Some(toi) = ray.intersect_plane(Vec3::ZERO, Vec3::Y) else {
		return;
	};
	move_input_events.send(MoveInputEvent(ray.origin + ray.direction * toi));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::MoveInputEvent;
	use bevy::{
		app::{App, Update},
		ecs::event::Events,
		math::Ray,
	};
	use common::test_tools::utils::SingleThreadedApp;

	fn setup(cam_ray: Option<Ray>) -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, trigger_move_input_event);
		app.add_event::<MoveInputEvent>();
		app.init_resource::<Input<MouseButton>>();
		app.insert_resource(CamRay(cam_ray));

		app
	}

	fn move_input_events(app: &App) -> Vec<MoveInputEvent> {
		let events = app.world.resource::<Events<MoveInputEvent>>();
		let mut reader = events.get_reader();

		reader.read(events).cloned().collect()
	}

	#[test]
	fn trigger_immediately_on_left_mouse_press() {
		let mut app = setup(Some(Ray {
			origin: Vec3::ONE,
			direction: Vec3::NEG_Y,
		}));
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);

		app.update();

		assert_eq!(
			vec![MoveInputEvent(Vec3::new(1., 0., 1.))],
			move_input_events(&app)
		);
	}

	#[test]
	fn no_event_when_other_mouse_button_pressed() {
		let mut app = setup(Some(Ray::default()));
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Middle);

		app.update();

		assert_eq!(vec![] as Vec<MoveInputEvent>, move_input_events(&app));
	}

	#[test]
	fn no_event_when_no_ray() {
		let mut app = setup(None);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);

		app.update();

		assert_eq!(vec![] as Vec<MoveInputEvent>, move_input_events(&app));
	}

	#[test]
	fn use_ray_intersection_with_zero_elevation_plane() {
		let mut app = setup(Some(Ray {
			origin: Vec3::new(1., 4., 1.),
			direction: Vec3::new(3., -4., 0.).normalize(),
		}));
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);

		app.update();

		assert_eq!(
			vec![MoveInputEvent(Vec3::new(4., 0., 1.))],
			move_input_events(&app)
		);
	}
}
