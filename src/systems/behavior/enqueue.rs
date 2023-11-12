use crate::{
	behaviors::Behavior,
	components::{Queue, Schedule, ScheduleMode, SlotKey, WaitNext},
	traits::get_ray::GetRayFromCamera,
};
use bevy::{
	prelude::{Camera, Commands, Entity, GlobalTransform, Query, Ray},
	window::Window,
};

pub fn enqueue<TGetRay: GetRayFromCamera>(
	camera: Query<(&Camera, &GlobalTransform)>,
	window: Query<&Window>,
	mut agents: Query<(Entity, &Schedule, &mut Queue)>,
	mut commands: Commands,
) {
	if agents.is_empty() {
		return;
	}

	let (camera, camera_transform) = camera.single();
	let window = window.single();
	let ray = TGetRay::get_ray(camera, camera_transform, window);

	for (agent, schedule, mut queue) in &mut agents {
		enqueue_agent_behaviors(agent, schedule, &mut queue, &mut commands, ray);
		commands.entity(agent).remove::<Schedule>();
	}
}

fn enqueue_agent_behaviors(
	agent: Entity,
	schedule: &Schedule,
	queue: &mut Queue,
	commands: &mut Commands,
	ray: Option<Ray>,
) {
	for behavior_slot in &schedule.behaviors {
		enqueue_agent_behavior(agent, queue, schedule.mode, behavior_slot, commands, ray);
	}
}

fn enqueue_agent_behavior(
	agent: Entity,
	queue: &mut Queue,
	schedule_mode: ScheduleMode,
	behavior_slot: (&SlotKey, &Behavior),
	commands: &mut Commands,
	ray: Option<Ray>,
) {
	let (_, behavior) = behavior_slot;

	let Some(behavior_and_ray) = ray.map(|r| (*behavior, r)) else {
		return;
	};

	match schedule_mode {
		ScheduleMode::Enqueue => queue.0.push_back(behavior_and_ray),
		ScheduleMode::Override => {
			queue.0.clear();
			queue.0.push_back(behavior_and_ray);
			commands.entity(agent).insert(WaitNext);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Schedule, ScheduleMode, Side, WaitNext};
	use bevy::{
		ecs::system::EntityCommands,
		prelude::{App, Camera, Camera3dBundle, GlobalTransform, Ray, Update, Vec3},
		utils::default,
		window::Window,
	};
	use mockall::automock;

	const TEST_RAY: Ray = Ray {
		origin: Vec3::ONE,
		direction: Vec3::Z,
	};

	struct GetTestRay;

	impl GetRayFromCamera for GetTestRay {
		fn get_ray(
			_camera: &Camera,
			_camera_transform: &GlobalTransform,
			_window: &Window,
		) -> Option<Ray> {
			Some(TEST_RAY)
		}
	}

	fn fake_behavior_insert<const T: char>(_entity: &mut EntityCommands, _ray: Ray) {}

	fn setup<TGetRay: GetRayFromCamera + 'static>() -> App {
		let mut app = App::new();

		app.world.spawn(Camera3dBundle {
			camera: Camera {
				order: 42,
				..default()
			},
			global_transform: GlobalTransform::from_xyz(4., 3., 2.),
			..default()
		});
		app.world.spawn(Window {
			title: "Window".to_owned(),
			..default()
		});
		app.add_systems(Update, enqueue::<TGetRay>);

		app
	}

	#[test]
	fn set_enqueue() {
		let mut app = setup::<GetTestRay>();
		let agent = app
			.world
			.spawn((
				Schedule {
					mode: ScheduleMode::Enqueue,
					behaviors: [(
						SlotKey::Hand(Side::Left),
						Behavior {
							insert_fn: fake_behavior_insert::<'c'>,
						},
					)]
					.into(),
				},
				Queue(
					[
						(
							Behavior {
								insert_fn: fake_behavior_insert::<'a'>,
							},
							Ray::default(),
						),
						(
							Behavior {
								insert_fn: fake_behavior_insert::<'c'>,
							},
							Ray::default(),
						),
					]
					.into(),
				),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			vec![
				&(
					Behavior {
						insert_fn: fake_behavior_insert::<'a'>,
					},
					Ray::default(),
				),
				&(
					Behavior {
						insert_fn: fake_behavior_insert::<'c'>,
					},
					Ray::default(),
				),
				&(
					Behavior {
						insert_fn: fake_behavior_insert::<'c'>,
					},
					TEST_RAY,
				),
			],
			queue.0.iter().collect::<Vec<&(Behavior, Ray)>>()
		);
	}

	#[test]
	fn set_override() {
		let mut app = setup::<GetTestRay>();
		let agent = app
			.world
			.spawn((
				Schedule {
					mode: ScheduleMode::Override,
					behaviors: [(
						SlotKey::Hand(Side::Left),
						Behavior {
							insert_fn: fake_behavior_insert::<'c'>,
						},
					)]
					.into(),
				},
				Queue(
					[
						(
							Behavior {
								insert_fn: fake_behavior_insert::<'a'>,
							},
							Ray::default(),
						),
						(
							Behavior {
								insert_fn: fake_behavior_insert::<'c'>,
							},
							Ray::default(),
						),
					]
					.into(),
				),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(
				vec![&(
					Behavior {
						insert_fn: fake_behavior_insert::<'c'>,
					},
					TEST_RAY,
				),],
				true
			),
			(queue.0.iter().collect(), agent.contains::<WaitNext>())
		);
	}

	#[test]
	fn remove_schedule() {
		let mut app = setup::<GetTestRay>();
		let agent = app
			.world
			.spawn((
				Schedule {
					mode: ScheduleMode::Override,
					behaviors: [(
						SlotKey::Hand(Side::Left),
						Behavior {
							insert_fn: fake_behavior_insert::<'c'>,
						},
					)]
					.into(),
				},
				Queue([].into()),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Schedule>());
	}

	#[test]
	fn ray_from_camera_and_window() {
		struct _GetRay;

		#[automock]
		impl GetRayFromCamera for _GetRay {
			fn get_ray(
				_camera: &Camera,
				_camera_transform: &GlobalTransform,
				_window: &Window,
			) -> Option<Ray> {
				None
			}
		}

		let mut app = setup::<Mock_GetRay>();
		let get_ray = Mock_GetRay::get_ray_context();
		let ray = Ray {
			origin: Vec3::ZERO,
			direction: Vec3::ONE,
		};

		get_ray
			.expect()
			.withf(|cam, cam_transform, window| {
				*cam_transform == GlobalTransform::from_xyz(4., 3., 2.)
				// using specific values for non-equatable variables
				&& cam.order == 42 && window.title == "Window"
			})
			.times(1)
			.return_const(ray);

		app.world.spawn((
			Schedule {
				mode: ScheduleMode::Override,
				behaviors: [(
					SlotKey::Hand(Side::Left),
					Behavior {
						insert_fn: fake_behavior_insert::<'c'>,
					},
				)]
				.into(),
			},
			Queue([].into()),
		));

		app.update();
	}

	#[test]
	fn do_not_produce_ray_when_nothing_scheduled() {
		struct _GetRay;

		#[automock]
		impl GetRayFromCamera for _GetRay {
			fn get_ray(
				_camera: &Camera,
				_camera_transform: &GlobalTransform,
				_window: &Window,
			) -> Option<Ray> {
				None
			}
		}

		let mut app = setup::<Mock_GetRay>();
		let get_ray = Mock_GetRay::get_ray_context();
		let ray = Ray {
			origin: Vec3::ZERO,
			direction: Vec3::ONE,
		};

		get_ray.expect().times(0).return_const(ray);

		app.world.spawn(Queue([].into()));

		app.update();
	}
}
