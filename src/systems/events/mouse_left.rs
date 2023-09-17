use crate::traits::get_ray::GetRayFromCamera;
use crate::traits::new::New1;
use bevy::prelude::*;

pub fn mouse_left<TTools: GetRayFromCamera, TEvent: New1<Vec3> + Event>(
	mouse: Res<Input<MouseButton>>,
	windows: Query<&Window>,
	query: Query<(&Camera, &GlobalTransform)>,
	mut event_writer: EventWriter<TEvent>,
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

	event_writer.send(TEvent::new(ray.origin + ray.direction * distance));
}

#[cfg(test)]
mod tests {
	use mockall::automock;

	use super::*;

	#[derive(Event)]
	struct _Event {
		pub vec: Vec3,
	}

	impl New1<Vec3> for _Event {
		fn new(value: Vec3) -> Self {
			Self { vec: value }
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

		app.add_systems(Update, mouse_left::<Mock_Tools, _Event>);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);
		app.update();

		let event_rs = app.world.resource::<Events<_Event>>();
		let event_rd = event_rs.get_reader();

		assert_eq!(1, event_rd.len(event_rs));
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

		app.add_systems(Update, mouse_left::<Mock_Tools, _Event>);
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

		app.add_systems(Update, mouse_left::<Mock_Tools, _Event>);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);
		app.update();

		let event_rs = app.world.resource::<Events<_Event>>();
		let mut event_rd = event_rs.get_reader();
		let events: Vec<&_Event> = event_rd.iter(event_rs).collect();
		let event = events.first().unwrap();

		assert_eq!(Vec3::new(1., 0., 1.), event.vec);
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

		app.add_systems(Update, mouse_left::<Mock_Tools, _Event>);
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
		app.add_systems(Update, mouse_left::<Mock_Tools, _Event>);
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
		app.add_systems(Update, mouse_left::<Mock_Tools, _Event>);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);
		app.update();

		let event_rs = app.world.resource::<Events<_Event>>();
		let event_rd = event_rs.get_reader();

		assert!(event_rd.is_empty(event_rs));
	}
}
