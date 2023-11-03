use crate::{
	components::{GetBehaviorFn, Idle, Queue, Schedule, ScheduleMode},
	traits::get_ray::GetRayFromCamera,
};
use bevy::{
	prelude::{Camera, Commands, Component, Entity, GlobalTransform, Query, Ray, With},
	window::Window,
};

fn schedule_agent_behavior<TBehavior>(
	agent: Entity,
	queue: &mut Queue<TBehavior>,
	schedule_mode: ScheduleMode,
	get_behavior: &GetBehaviorFn<TBehavior>,
	commands: &mut Commands,
	ray: Option<Ray>,
) {
	let Some(behavior) = ray.and_then(get_behavior) else {
		return;
	};

	match schedule_mode {
		ScheduleMode::Enqueue => queue.0.push_back(behavior),
		ScheduleMode::Override => {
			queue.0.clear();
			queue.0.push_back(behavior);
			commands.entity(agent).insert(Idle);
		}
	}
}

fn schedule_agent_behaviors<TBehavior>(
	agent: Entity,
	schedule: &Schedule<TBehavior>,
	queue: &mut Queue<TBehavior>,
	commands: &mut Commands,
	ray: Option<Ray>,
) {
	for get_behavior in &schedule.get_behaviors {
		schedule_agent_behavior(agent, queue, schedule.mode, get_behavior, commands, ray);
	}
}

type Components<'a, TBehavior> = (Entity, &'a Schedule<TBehavior>, &'a mut Queue<TBehavior>);

pub fn schedule_behaviors<
	TAgent: Component,
	TBehavior: Sync + Send + 'static,
	TGetRay: GetRayFromCamera,
>(
	camera: Query<(&Camera, &GlobalTransform)>,
	window: Query<&Window>,
	mut agents: Query<Components<TBehavior>, With<TAgent>>,
	mut commands: Commands,
) {
	if agents.is_empty() {
		return;
	}

	let (camera, camera_transform) = camera.single();
	let window = window.single();
	let ray = TGetRay::get_ray(camera, camera_transform, window);

	for (agent, schedule, mut queue) in &mut agents {
		schedule_agent_behaviors(agent, schedule, &mut queue, &mut commands, ray);
		commands.entity(agent).remove::<Schedule<TBehavior>>();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Idle, Schedule, ScheduleMode};
	use bevy::{
		prelude::{App, Camera, Camera3dBundle, GlobalTransform, Ray, Update, Vec3},
		utils::default,
		window::Window,
	};
	use mockall::automock;

	#[derive(Component)]
	struct Agent;

	#[derive(PartialEq, Debug)]
	struct MockBehavior {
		pub ray: Ray,
	}

	const DEFAULT_RAY: Ray = Ray {
		origin: Vec3::ONE,
		direction: Vec3::Z,
	};

	struct GetDefaultRay;

	impl GetRayFromCamera for GetDefaultRay {
		fn get_ray(
			_camera: &Camera,
			_camera_transform: &GlobalTransform,
			_window: &Window,
		) -> Option<Ray> {
			Some(DEFAULT_RAY)
		}
	}

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
		app.add_systems(Update, schedule_behaviors::<Agent, MockBehavior, TGetRay>);

		app
	}

	#[test]
	fn set_enqueue() {
		let mut app = setup::<GetDefaultRay>();
		let agent = app
			.world
			.spawn((
				Agent,
				Schedule {
					mode: ScheduleMode::Enqueue,
					get_behaviors: vec![|ray| Some(MockBehavior { ray })],
				},
				Queue(
					vec![
						MockBehavior {
							ray: Ray {
								origin: Vec3::Z,
								direction: Vec3::Y,
							},
						},
						MockBehavior {
							ray: Ray {
								origin: Vec3::Z,
								direction: Vec3::Y,
							},
						},
					]
					.into(),
				),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<MockBehavior>>().unwrap();

		assert_eq!(
			vec![
				&MockBehavior {
					ray: Ray {
						origin: Vec3::Z,
						direction: Vec3::Y,
					},
				},
				&MockBehavior {
					ray: Ray {
						origin: Vec3::Z,
						direction: Vec3::Y,
					},
				},
				&MockBehavior { ray: DEFAULT_RAY }
			],
			queue.0.iter().collect::<Vec<&MockBehavior>>()
		);
	}

	#[test]
	fn set_override() {
		let mut app = setup::<GetDefaultRay>();
		let agent = app
			.world
			.spawn((
				Agent,
				Schedule {
					mode: ScheduleMode::Override,
					get_behaviors: vec![|ray| Some(MockBehavior { ray })],
				},
				Queue(
					vec![
						MockBehavior {
							ray: Ray {
								origin: Vec3::Z,
								direction: Vec3::Y,
							},
						},
						MockBehavior {
							ray: Ray {
								origin: Vec3::Z,
								direction: Vec3::Y,
							},
						},
					]
					.into(),
				),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<MockBehavior>>().unwrap();

		assert_eq!(
			(vec![&MockBehavior { ray: DEFAULT_RAY }], true),
			(
				queue.0.iter().collect::<Vec<&MockBehavior>>(),
				agent.contains::<Idle>()
			)
		);
	}

	#[test]
	fn remove_schedule() {
		let mut app = setup::<GetDefaultRay>();
		let agent = app
			.world
			.spawn((
				Agent,
				Schedule {
					mode: ScheduleMode::Override,
					get_behaviors: vec![|ray| Some(MockBehavior { ray })],
				},
				Queue::<MockBehavior>::new(),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Schedule<MockBehavior>>());
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
			Agent,
			Schedule {
				mode: ScheduleMode::Override,
				get_behaviors: vec![|ray| Some(MockBehavior { ray })],
			},
			Queue::<MockBehavior>::new(),
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

		app.world.spawn((Agent, Queue::<MockBehavior>::new()));

		app.update();
	}
}