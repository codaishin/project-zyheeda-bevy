use crate::traits::new::New;
use crate::traits::set_world_position_from_ray::SetWorldPositionFromRay;
use bevy::prelude::*;

pub fn send_move_command<TWorldPositionEvent: SetWorldPositionFromRay + New + Event>(
	get_ray: impl Fn(&Window, &Camera, &GlobalTransform) -> Option<Ray>,
) -> Box<
	impl Fn(
		Res<Input<MouseButton>>,
		Query<&Window>,
		Query<(&Camera, &GlobalTransform)>,
		EventWriter<TWorldPositionEvent>,
	),
> {
	Box::new(
		move |mouse: Res<Input<MouseButton>>,
		      windows: Query<&Window>,
		      query: Query<(&Camera, &GlobalTransform)>,
		      mut event_writer: EventWriter<TWorldPositionEvent>| {
			if !mouse.just_pressed(MouseButton::Left) {
				return;
			}
			let Ok((cam, transform)) = query.get_single() else {
				return; // FIXME: Handle properly
			};
			let Ok(window) = windows.get_single() else {
				return; // FIXME: Handle properly
			};
			let Some(ray) = get_ray(window, cam, transform) else {
				return;
			};

			let mut event = TWorldPositionEvent::new();
			event.set_world_position(ray);
			event_writer.send(event);
		},
	)
}

pub fn get_ray(window: &Window, camera: &Camera, transform: &GlobalTransform) -> Option<Ray> {
	window
		.cursor_position()
		.and_then(|c| camera.viewport_to_world(transform, c))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Event)]
	struct _Event {
		pub called_with_rays: Vec<Ray>,
	}

	impl New for _Event {
		fn new() -> Self {
			Self {
				called_with_rays: vec![],
			}
		}
	}

	impl SetWorldPositionFromRay for _Event {
		fn set_world_position(&mut self, ray: Ray) {
			self.called_with_rays.push(ray)
		}
	}

	fn setup_app() -> (App, Entity, GlobalTransform, Entity) {
		let mut app = App::new();
		let input = Input::<MouseButton>::default();
		let window = Window {
			title: "test window".to_string(),
			..default()
		};
		let transform = Transform::from_xyz(1., 2., 3.);
		let cam_transform = GlobalTransform::from(transform);
		let cam = Camera3dBundle {
			transform,
			global_transform: cam_transform,
			camera: Camera {
				order: 42,
				..default()
			},
			..default()
		};
		let cam_id = app.world.spawn(cam).id();
		let window_id = app.world.spawn(window).id();

		app.add_event::<_Event>();
		app.insert_resource(input);

		(app, cam_id, cam_transform, window_id)
	}

	#[test]
	fn send_event_with_ray() {
		let (mut app, ..) = setup_app();
		let expected_ray = Ray {
			origin: Vec3::ONE,
			direction: Vec3::Z,
		};

		app.add_systems(
			Update,
			send_move_command::<_Event>(move |_, _, _| Some(expected_ray)),
		);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);
		app.update();

		let event_resource = app.world.resource::<Events<_Event>>();
		let mut event_reader = event_resource.get_reader();
		let events: Vec<&_Event> = event_reader.iter(event_resource).collect();
		let [event] = events.as_slice() else {
			panic!("got {} events, expected 1", events.len());
		};
		let [ray] = event.called_with_rays.as_slice() else {
			panic!("got {} ray calls, expected 1", event.called_with_rays.len())
		};

		assert_eq!(expected_ray, *ray)
	}

	#[test]
	fn no_event_when_no_input() {
		let (mut app, ..) = setup_app();

		app.add_systems(
			Update,
			send_move_command::<_Event>(|_, _, _| {
				Some(Ray {
					origin: Vec3::ZERO,
					direction: Vec3::ONE,
				})
			}),
		);
		app.update();

		let event_resource = app.world.resource::<Events<_Event>>();
		let mut event_reader = event_resource.get_reader();
		let events: Vec<&_Event> = event_reader.iter(event_resource).collect();

		assert_eq!(0, events.len())
	}

	#[test]
	fn no_event_when_no_ray() {
		let (mut app, ..) = setup_app();

		app.add_systems(Update, send_move_command::<_Event>(|_, _, _| None));
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);
		app.update();

		let event_resource = app.world.resource::<Events<_Event>>();
		let mut event_reader = event_resource.get_reader();
		let events: Vec<&_Event> = event_reader.iter(event_resource).collect();

		assert_eq!(0, events.len())
	}

	#[test]
	fn call_get_cursor_pos_with_correct_args() {
		let (mut app, cam_id, cam_transform, window_id) = setup_app();

		// using fields as comparison for non equatable structs
		let window_title = app.world.get::<Window>(window_id).unwrap().title.to_owned();
		let camera_order = app.world.get::<Camera>(cam_id).unwrap().order;

		app.add_systems(
			Update,
			send_move_command::<_Event>(move |w, c, c_t| {
				assert_eq!(
					(window_title.to_owned(), camera_order, cam_transform),
					(w.title.to_owned(), c.order, *c_t)
				);
				None
			}),
		);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);
		app.update();
	}
}
