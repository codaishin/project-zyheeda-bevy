use bevy::prelude::*;
use common::{
	tools::action_key::{movement::MovementKey, user_input::UserInput},
	traits::{intersect_at::IntersectAt, key_mappings::Pressed},
};

impl<T> TriggerPointerMovement for T where T: From<Vec3> + Event {}

pub(crate) trait TriggerPointerMovement: From<Vec3> + Event {
	fn trigger_pointer_movement<TRay, TMap>(
		input: Res<ButtonInput<UserInput>>,
		map: Res<TMap>,
		cam_ray: Res<TRay>,
		mut move_input_events: EventWriter<Self>,
	) where
		TRay: IntersectAt + Resource,
		TMap: Pressed<MovementKey> + Resource,
	{
		if !map.pressed(&input).any(|key| key == MovementKey::Pointer) {
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
		traits::{intersect_at::IntersectAt, iteration::IterFinite, nested_mock::NestedMocks},
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

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl Pressed<MovementKey> for _Map {
		fn pressed(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = MovementKey> {
			self.mock.pressed(input)
		}
	}

	fn setup(ray: _Ray, map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(ray);
		app.insert_resource(map);
		app.init_resource::<ButtonInput<UserInput>>();
		app.add_event::<_Event>();
		app.add_systems(Update, _Event::trigger_pointer_movement::<_Ray, _Map>);

		app
	}

	fn move_input_events(app: &App) -> Vec<_Event> {
		let events = app.world().resource::<Events<_Event>>();
		let mut cursor = events.get_cursor();

		cursor.read(events).copied().collect()
	}

	#[test]
	fn trigger_immediately_on_movement_pointer_press() {
		let mut app = setup(
			_Ray::new().with_mock(|mock| {
				mock.expect_intersect_at()
					.return_const(Vec3::new(1., 2., 3.));
			}),
			_Map::new().with_mock(|mock| {
				mock.expect_pressed()
					.returning(|_| Box::new(std::iter::once(MovementKey::Pointer)));
			}),
		);

		app.update();

		assert_eq!(vec![_Event(Vec3::new(1., 2., 3.))], move_input_events(&app));
	}

	#[test]
	fn no_event_when_other_movement_button_pressed() {
		let mut app = setup(
			_Ray::new().with_mock(|mock| {
				mock.expect_intersect_at().return_const(Vec3::default());
			}),
			_Map::new().with_mock(|mock| {
				mock.expect_pressed().returning(|_| {
					Box::new(MovementKey::iterator().filter(|key| key != &MovementKey::Pointer))
				});
			}),
		);

		app.update();

		assert_eq!(vec![] as Vec<_Event>, move_input_events(&app));
	}

	#[test]
	fn no_event_when_no_intersection() {
		let mut app = setup(
			_Ray::new().with_mock(|mock| {
				mock.expect_intersect_at().return_const(None);
			}),
			_Map::new().with_mock(|mock| {
				mock.expect_pressed()
					.returning(|_| Box::new(std::iter::once(MovementKey::Pointer)));
			}),
		);

		app.update();

		assert_eq!(vec![] as Vec<_Event>, move_input_events(&app));
	}

	#[test]
	fn call_intersect_with_height_zero() {
		let mut app = setup(
			_Ray::new().with_mock(|mock| {
				mock.expect_intersect_at()
					.with(eq(0.))
					.times(1)
					.return_const(None);
			}),
			_Map::new().with_mock(|mock| {
				mock.expect_pressed()
					.returning(|_| Box::new(std::iter::once(MovementKey::Pointer)));
			}),
		);

		app.update();
	}

	#[test]
	fn call_map_with_correct_input() {
		let mut input = ButtonInput::default();
		input.press(UserInput::MouseButton(MouseButton::Back));
		let mut app = setup(
			_Ray::new().with_mock(|mock| {
				mock.expect_intersect_at().return_const(None);
			}),
			_Map::new().with_mock(|mock| {
				mock.expect_pressed().returning(|input| {
					assert_eq!(
						vec![&UserInput::MouseButton(MouseButton::Back)],
						input.get_pressed().collect::<Vec<_>>()
					);
					Box::new(std::iter::empty())
				});
			}),
		);
		app.insert_resource(input);

		app.update();
	}
}
