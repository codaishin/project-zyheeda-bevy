use crate::events::MoveInputEvent;
use bevy::{
	ecs::{
		event::EventWriter,
		system::{Res, Resource},
	},
	input::{mouse::MouseButton, Input},
};
use common::traits::intersect_at::IntersectAt;

pub(crate) fn trigger_move_input_event<TRay: IntersectAt + Resource>(
	mouse_input: Res<Input<MouseButton>>,
	cam_ray: Res<TRay>,
	mut move_input_events: EventWriter<MoveInputEvent>,
) {
	if !mouse_input.pressed(MouseButton::Left) {
		return;
	}
	let Some(intersection) = cam_ray.intersect_at(0.) else {
		return;
	};
	move_input_events.send(MoveInputEvent(intersection));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::MoveInputEvent;
	use bevy::{
		app::{App, Update},
		ecs::event::Events,
		math::Vec3,
	};
	use common::{test_tools::utils::SingleThreadedApp, traits::intersect_at::IntersectAt};
	use mockall::{automock, predicate::eq};

	#[derive(Resource, Default)]
	struct _Ray {
		mock: Mock_Ray,
	}

	#[automock]
	impl IntersectAt for _Ray {
		fn intersect_at(&self, height: f32) -> Option<Vec3> {
			self.mock.intersect_at(height)
		}
	}

	fn setup(ray: _Ray) -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, trigger_move_input_event::<_Ray>);
		app.add_event::<MoveInputEvent>();
		app.init_resource::<Input<MouseButton>>();
		app.insert_resource(ray);

		app
	}

	fn move_input_events(app: &App) -> Vec<MoveInputEvent> {
		let events = app.world.resource::<Events<MoveInputEvent>>();
		let mut reader = events.get_reader();

		reader.read(events).cloned().collect()
	}

	#[test]
	fn trigger_immediately_on_left_mouse_press() {
		let mut ray = _Ray::default();
		ray.mock
			.expect_intersect_at()
			.return_const(Vec3::new(1., 2., 3.));
		let mut app = setup(ray);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);

		app.update();

		assert_eq!(
			vec![MoveInputEvent(Vec3::new(1., 2., 3.))],
			move_input_events(&app)
		);
	}

	#[test]
	fn no_event_when_other_mouse_button_pressed() {
		let mut ray = _Ray::default();
		ray.mock.expect_intersect_at().return_const(Vec3::default());
		let mut app = setup(ray);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Middle);

		app.update();

		assert_eq!(vec![] as Vec<MoveInputEvent>, move_input_events(&app));
	}

	#[test]
	fn no_event_when_no_intersection() {
		let mut ray = _Ray::default();
		ray.mock.expect_intersect_at().return_const(None);
		let mut app = setup(ray);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);

		app.update();

		assert_eq!(vec![] as Vec<MoveInputEvent>, move_input_events(&app));
	}

	#[test]
	fn call_intersect_with_height_zero() {
		let mut ray = _Ray::default();
		ray.mock
			.expect_intersect_at()
			.with(eq(0.))
			.times(1)
			.return_const(None);
		let mut app = setup(ray);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);

		app.update();
	}
}
