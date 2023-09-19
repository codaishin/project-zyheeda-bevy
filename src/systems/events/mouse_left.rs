use crate::traits::get_ray::GetRayFromCamera;
use bevy::prelude::*;

pub fn mouse_left<
	TTools: GetRayFromCamera,
	TEvent: From<Vec3> + Event,
	TEnqueueEvent: From<Vec3> + Event,
>(
	mouse: Res<Input<MouseButton>>,
	keys: Res<Input<KeyCode>>,
	windows: Query<&Window>,
	query: Query<(&Camera, &GlobalTransform)>,
	mut event_writer: EventWriter<TEvent>,
	mut queue_event_writer: EventWriter<TEnqueueEvent>,
) {
	if !mouse.just_pressed(MouseButton::Left) {
		return;
	}
	let Ok((camera, transform)) = query.get_single() else {
		return; // FIXME: Handle properly
	};
	let Ok(window) = windows.get_single() else {
		return; // FIXME: Handle properly
	};
	let Some(ray) = TTools::get_ray(camera, transform, window) else {
		return;
	};
	let Some(distance) = ray.intersect_plane(Vec3::ZERO, Vec3::Y) else {
		return;
	};

	let target = ray.origin + ray.direction * distance;

	if keys.pressed(KeyCode::ShiftLeft) {
		queue_event_writer.send(target.into());
	} else {
		event_writer.send(target.into());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use mockall::automock;

	#[derive(Event)]
	struct _Event {
		pub vec: Vec3,
	}

	impl From<Vec3> for _Event {
		fn from(vec: Vec3) -> Self {
			Self { vec }
		}
	}

	#[derive(Event)]
	struct _EnqueueEvent {
		pub vec: Vec3,
	}

	impl From<Vec3> for _EnqueueEvent {
		fn from(vec: Vec3) -> Self {
			Self { vec }
		}
	}

	fn setup_app() -> (App, Entity, GlobalTransform, Entity) {
		let mut app = App::new();
		let mouse = Input::<MouseButton>::default();
		let keys = Input::<KeyCode>::default();
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
		app.add_event::<_EnqueueEvent>();
		app.insert_resource(mouse);
		app.insert_resource(keys);

		(app, cam_id, cam_transform, window_id)
	}

	#[test]
	fn send_event() {
		let (mut app, ..) = setup_app();
		let get_ray = Mock_Tools::get_ray_context();

		struct _Tools;

		#[automock]
		impl GetRayFromCamera for _Tools {
			fn get_ray(
				_camera: &Camera,
				_camera_transform: &GlobalTransform,
				_window: &Window,
			) -> Option<Ray> {
				None
			}
		}

		get_ray.expect().times(1).return_const(Some(Ray {
			origin: Vec3::Y,
			direction: -Vec3::Y,
		}));

		app.add_systems(Update, mouse_left::<Mock_Tools, _Event, _EnqueueEvent>);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);
		app.update();

		let event_rs = app.world.resource::<Events<_Event>>();
		let event_rd = event_rs.get_reader();
		let queue_event_rs = app.world.resource::<Events<_EnqueueEvent>>();
		let queue_event_rd = queue_event_rs.get_reader();

		assert_eq!(
			(1, 0),
			(event_rd.len(event_rs), queue_event_rd.len(queue_event_rs))
		);
	}

	#[test]
	fn send_enqueue_event() {
		let (mut app, ..) = setup_app();
		let get_ray = Mock_Tools::get_ray_context();

		struct _Tools;

		#[automock]
		impl GetRayFromCamera for _Tools {
			fn get_ray(
				_camera: &Camera,
				_camera_transform: &GlobalTransform,
				_window: &Window,
			) -> Option<Ray> {
				None
			}
		}

		get_ray.expect().times(1).return_const(Some(Ray {
			origin: Vec3::Y,
			direction: -Vec3::Y,
		}));

		app.add_systems(Update, mouse_left::<Mock_Tools, _Event, _EnqueueEvent>);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);
		app.world
			.resource_mut::<Input<KeyCode>>()
			.press(KeyCode::ShiftLeft);
		app.update();

		let event_rs = app.world.resource::<Events<_Event>>();
		let event_rd = event_rs.get_reader();
		let queue_event_rs = app.world.resource::<Events<_EnqueueEvent>>();
		let queue_event_rd = queue_event_rs.get_reader();

		assert_eq!(
			(0, 1),
			(event_rd.len(event_rs), queue_event_rd.len(queue_event_rs))
		);
	}

	#[test]
	fn get_ray_args() {
		let (mut app, cam_id, camera_transform, window_id) = setup_app();
		let get_ray = Mock_Tools::get_ray_context();
		let window_title = app.world.get::<Window>(window_id).unwrap().title.to_owned();
		let camera_order = app.world.get::<Camera>(cam_id).unwrap().order;

		struct _Tools;

		#[automock]
		impl GetRayFromCamera for _Tools {
			fn get_ray(
				_camera: &Camera,
				_camera_transform: &GlobalTransform,
				_window: &Window,
			) -> Option<Ray> {
				None
			}
		}

		get_ray
			.expect()
			.withf(move |arg_camera, arg_camera_transform, arg_window| {
				// using fields values as comparison for non equatable structs Camera and Window
				arg_camera.order == camera_order
					&& *arg_camera_transform == camera_transform
					&& arg_window.title == window_title
			})
			.times(1)
			.return_const(Some(Ray {
				origin: Vec3::ZERO,
				direction: Vec3::X,
			}));

		app.add_systems(Update, mouse_left::<Mock_Tools, _Event, _EnqueueEvent>);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);
		app.update();
	}

	#[test]
	fn send_event_with_correct_target() {
		let (mut app, ..) = setup_app();
		let get_ray = Mock_Tools::get_ray_context();

		struct _Tools;

		#[automock]
		impl GetRayFromCamera for _Tools {
			fn get_ray(
				_camera: &Camera,
				_camera_transform: &GlobalTransform,
				_window: &Window,
			) -> Option<Ray> {
				None
			}
		}

		get_ray.expect().times(1).return_const(Some(Ray {
			origin: Vec3::new(1., 3., 1.),
			direction: -Vec3::Y,
		}));

		app.add_systems(Update, mouse_left::<Mock_Tools, _Event, _EnqueueEvent>);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);
		app.update();

		let event_rs = app.world.resource::<Events<_Event>>();
		let mut event_rd = event_rs.get_reader();
		let events: Vec<Vec3> = event_rd.iter(event_rs).map(|e| e.vec).collect();

		assert_eq!(vec![Vec3::new(1., 0., 1.)], events);
	}
	#[test]
	fn send_enqueue_event_with_correct_target() {
		let (mut app, ..) = setup_app();
		let get_ray = Mock_Tools::get_ray_context();

		struct _Tools;

		#[automock]
		impl GetRayFromCamera for _Tools {
			fn get_ray(
				_camera: &Camera,
				_camera_transform: &GlobalTransform,
				_window: &Window,
			) -> Option<Ray> {
				None
			}
		}

		get_ray.expect().times(1).return_const(Some(Ray {
			origin: Vec3::new(1., 3., 1.),
			direction: -Vec3::Y,
		}));

		app.add_systems(Update, mouse_left::<Mock_Tools, _Event, _EnqueueEvent>);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);
		app.world
			.resource_mut::<Input<KeyCode>>()
			.press(KeyCode::ShiftLeft);
		app.update();

		let event_rs = app.world.resource::<Events<_EnqueueEvent>>();
		let mut event_rd = event_rs.get_reader();
		let targets: Vec<Vec3> = event_rd.iter(event_rs).map(|e| e.vec).collect();

		assert_eq!(vec![Vec3::new(1., 0., 1.)], targets);
	}

	#[test]
	fn no_send_when_not_left_mouse_clicked() {
		let (mut app, ..) = setup_app();
		let get_ray = Mock_Tools::get_ray_context();

		struct _Tools;

		#[automock]
		impl GetRayFromCamera for _Tools {
			fn get_ray(
				_camera: &Camera,
				_camera_transform: &GlobalTransform,
				_window: &Window,
			) -> Option<Ray> {
				None
			}
		}

		get_ray.expect().return_const(Some(Ray {
			origin: Vec3::Y,
			direction: -Vec3::Y,
		}));

		app.add_systems(Update, mouse_left::<Mock_Tools, _Event, _EnqueueEvent>);
		app.update();

		let event_rs = app.world.resource::<Events<_Event>>();
		let event_rd = event_rs.get_reader();

		assert!(event_rd.is_empty(event_rs));
	}

	#[test]
	fn no_send_when_not_left_mouse_just_clicked() {
		let (mut app, ..) = setup_app();
		let get_ray = Mock_Tools::get_ray_context();

		struct _Tools;

		#[automock]
		impl GetRayFromCamera for _Tools {
			fn get_ray(
				_camera: &Camera,
				_camera_transform: &GlobalTransform,
				_window: &Window,
			) -> Option<Ray> {
				None
			}
		}

		get_ray.expect().times(1).return_const(Some(Ray {
			origin: Vec3::Y,
			direction: -Vec3::Y,
		}));
		app.add_systems(Update, mouse_left::<Mock_Tools, _Event, _EnqueueEvent>);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);
		app.update();
		app.world.resource_mut::<Events<_Event>>().clear();
		app.world
			.resource_mut::<Input<MouseButton>>()
			.clear_just_pressed(MouseButton::Left);
		app.update();

		let event_rs = app.world.resource::<Events<_Event>>();
		let event_rd = event_rs.get_reader();

		assert!(event_rd.is_empty(event_rs));
	}

	#[test]
	fn no_send_when_no_ray() {
		let (mut app, ..) = setup_app();
		let get_ray = Mock_Tools::get_ray_context();

		struct _Tools;

		#[automock]
		impl GetRayFromCamera for _Tools {
			fn get_ray(
				_camera: &Camera,
				_camera_transform: &GlobalTransform,
				_window: &Window,
			) -> Option<Ray> {
				None
			}
		}

		get_ray.expect().return_const(None);
		app.add_systems(Update, mouse_left::<Mock_Tools, _Event, _EnqueueEvent>);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);
		app.update();

		let event_rs = app.world.resource::<Events<_Event>>();
		let event_rd = event_rs.get_reader();

		assert!(event_rd.is_empty(event_rs));
	}
}
