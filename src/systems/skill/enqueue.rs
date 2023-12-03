use crate::{
	components::{Active, Queue, Queued, Schedule, ScheduleMode, Skill, SlotKey, WaitNext},
	traits::get_ray::GetRayFromCamera,
};
use bevy::{
	prelude::{Camera, Commands, Entity, GlobalTransform, Mut, Query, Ray},
	window::Window,
};

pub fn enqueue<TGetRay: GetRayFromCamera>(
	camera: Query<(&Camera, &GlobalTransform)>,
	window: Query<&Window>,
	mut agents: Query<(Entity, &Schedule, &mut Queue, Option<&mut Skill<Active>>)>,
	mut commands: Commands,
) {
	if agents.is_empty() {
		return;
	}

	let (camera, camera_transform) = camera.single();
	let window = window.single();
	let ray = TGetRay::get_ray(camera, camera_transform, window);

	for (agent, schedule, mut queue, mut running) in &mut agents {
		enqueue_skills(
			agent,
			schedule,
			&mut queue,
			&mut running,
			&mut commands,
			ray,
		);
		commands.entity(agent).remove::<Schedule>();
	}
}

fn enqueue_skills(
	agent: Entity,
	schedule: &Schedule,
	queue: &mut Queue,
	running: &mut Option<Mut<Skill<Active>>>,
	commands: &mut Commands,
	ray: Option<Ray>,
) {
	for slot in &schedule.skills {
		enqueue_skill(agent, schedule, queue, running, slot, commands, ray);
	}
}

fn enqueue_skill(
	agent: Entity,
	schedule: &Schedule,
	queue: &mut Queue,
	running: &mut Option<Mut<Skill<Active>>>,
	slot: (&SlotKey, &Skill),
	commands: &mut Commands,
	ray: Option<Ray>,
) {
	let (slot, skill) = slot;
	let slot = *slot;
	let Some(mut new) = ray.map(|ray| skill.with(Queued { ray, slot })) else {
		return;
	};
	let soft_override = running
		.as_mut()
		.map(|running| (new.marker.soft_override)(running, &mut new))
		.unwrap_or(false);

	match (schedule.mode, soft_override) {
		(ScheduleMode::Enqueue, ..) => {
			queue.0.push_back(new);
		}
		(ScheduleMode::Override, true) => {
			queue.0.clear();
			queue.0.push_back(new);
		}
		(ScheduleMode::Override, ..) => {
			queue.0.clear();
			queue.0.push_back(new);
			commands.entity(agent).insert(WaitNext);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Cast, Schedule, ScheduleMode, Side, WaitNext},
		markers::meta::MarkerMeta,
	};
	use bevy::{
		prelude::{App, Camera, Camera3dBundle, GlobalTransform, Ray, Update, Vec3},
		utils::default,
		window::Window,
	};
	use mockall::automock;
	use std::time::Duration;

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
		let new_skill = Skill {
			cast: Cast {
				pre: Duration::from_millis(100),
				..default()
			},
			..default()
		};
		let agent = app
			.world
			.spawn((
				Schedule {
					mode: ScheduleMode::Enqueue,
					skills: [(SlotKey::Hand(Side::Left), new_skill)].into(),
				},
				Queue(
					[
						Skill {
							cast: Cast {
								pre: Duration::from_millis(1),
								..default()
							},
							..default()
						},
						Skill {
							cast: Cast {
								pre: Duration::from_millis(2),
								..default()
							},
							..default()
						},
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
				&Skill {
					cast: Cast {
						pre: Duration::from_millis(1),
						..default()
					},
					..default()
				},
				&Skill {
					cast: Cast {
						pre: Duration::from_millis(2),
						..default()
					},
					..default()
				},
				&Skill {
					cast: Cast {
						pre: Duration::from_millis(100),
						..default()
					},
					data: Queued {
						ray: TEST_RAY,
						slot: SlotKey::Hand(Side::Left)
					},
					..default()
				},
			],
			queue.0.iter().collect::<Vec<&Skill<Queued>>>()
		);
	}

	#[test]
	fn set_override() {
		let mut app = setup::<GetTestRay>();
		let new_skill = Skill {
			cast: Cast {
				pre: Duration::from_millis(100),
				..default()
			},
			..default()
		};
		let agent = app
			.world
			.spawn((
				Schedule {
					mode: ScheduleMode::Override,
					skills: [(SlotKey::Hand(Side::Left), new_skill)].into(),
				},
				Queue(
					[
						Skill {
							cast: Cast {
								pre: Duration::from_millis(1),
								..default()
							},
							..default()
						},
						Skill {
							cast: Cast {
								pre: Duration::from_millis(2),
								..default()
							},
							..default()
						},
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
				vec![&new_skill.with(Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Left)
				})],
				true
			),
			(queue.0.iter().collect(), agent.contains::<WaitNext>())
		);
	}

	#[test]
	fn set_override_without_wait_next_when_soft_override_true() {
		let mut app = setup::<GetTestRay>();
		let new_skill = Skill {
			name: "new",
			cast: Cast {
				pre: Duration::from_millis(100),
				..default()
			},
			marker: MarkerMeta {
				soft_override: |running, new| running.name == "running" && new.name == "new",
				..default()
			},
			..default()
		};
		let agent = app
			.world
			.spawn((
				Skill {
					name: "running",
					data: Active {
						ray: TEST_RAY,
						slot: SlotKey::Hand(Side::Left),
						..default()
					},
					cast: Cast {
						pre: Duration::from_millis(1),
						..default()
					},
					..default()
				},
				Schedule {
					mode: ScheduleMode::Override,
					skills: [(SlotKey::Hand(Side::Left), new_skill)].into(),
				},
				Queue([].into()),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(
				vec![&new_skill.with(Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Left)
				})],
				false
			),
			(
				queue.0.iter().collect::<Vec<&Skill<Queued>>>(),
				agent.contains::<WaitNext>()
			)
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
					skills: [(SlotKey::Hand(Side::Left), Skill::default())].into(),
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
				skills: [(SlotKey::Hand(Side::Left), Skill::default())].into(),
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
