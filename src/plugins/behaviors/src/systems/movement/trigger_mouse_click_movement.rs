use bevy::prelude::*;
use common::{tools::keys::user_input::UserInput, traits::intersect_at::IntersectAt};

impl<T> TriggerMouseClickMovement for T where T: From<Vec3> + Event {}

pub(crate) trait TriggerMouseClickMovement: From<Vec3> + Event {
	fn trigger_mouse_click_movement<TRay: IntersectAt + Resource>(
		mouse_input: Res<ButtonInput<UserInput>>,
		cam_ray: Res<TRay>,
		mut move_input_events: EventWriter<Self>,
	) {
		if !mouse_input.pressed(UserInput::from(MouseButton::Left)) {
			return;
		}
		let Some(intersection) = cam_ray.intersect_at(0.) else {
			return;
		};
		move_input_events.send(Self::from(intersection));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::event::Events,
		math::Vec3,
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{intersect_at::IntersectAt, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Event, Debug, PartialEq, Clone, Copy)]
	struct _Event(Vec3);

	impl From<Vec3> for _Event {
		fn from(translation: Vec3) -> Self {
			Self(translation)
		}
	}

	#[derive(Resource, NestedMocks)]
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
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _Event::trigger_mouse_click_movement::<_Ray>);
		app.add_event::<_Event>();
		app.init_resource::<ButtonInput<UserInput>>();
		app.insert_resource(ray);

		app
	}

	fn move_input_events(app: &App) -> Vec<_Event> {
		let events = app.world().resource::<Events<_Event>>();
		let mut cursor = events.get_cursor();

		cursor.read(events).copied().collect()
	}

	#[test]
	fn trigger_immediately_on_left_mouse_press() {
		let mut app = setup(_Ray::new().with_mock(|mock| {
			mock.expect_intersect_at()
				.return_const(Vec3::new(1., 2., 3.));
		}));
		app.world_mut()
			.resource_mut::<ButtonInput<UserInput>>()
			.press(UserInput::from(MouseButton::Left));

		app.update();

		assert_eq!(vec![_Event(Vec3::new(1., 2., 3.))], move_input_events(&app));
	}

	#[test]
	fn no_event_when_other_mouse_button_pressed() {
		let mut app = setup(_Ray::new().with_mock(|mock| {
			mock.expect_intersect_at().return_const(Vec3::default());
		}));
		app.world_mut()
			.resource_mut::<ButtonInput<UserInput>>()
			.press(UserInput::from(MouseButton::Middle));

		app.update();

		assert_eq!(vec![] as Vec<_Event>, move_input_events(&app));
	}

	#[test]
	fn no_event_when_no_intersection() {
		let mut app = setup(_Ray::new().with_mock(|mock| {
			mock.expect_intersect_at().return_const(None);
		}));
		app.world_mut()
			.resource_mut::<ButtonInput<UserInput>>()
			.press(UserInput::from(MouseButton::Left));

		app.update();

		assert_eq!(vec![] as Vec<_Event>, move_input_events(&app));
	}

	#[test]
	fn call_intersect_with_height_zero() {
		let mut app = setup(_Ray::new().with_mock(|mock| {
			mock.expect_intersect_at()
				.with(eq(0.))
				.times(1)
				.return_const(None);
		}));
		app.world_mut()
			.resource_mut::<ButtonInput<UserInput>>()
			.press(UserInput::from(MouseButton::Left));

		app.update();
	}
}
