use crate::{
	components::{Active, Queue, Queued, Schedule, ScheduleMode, SlotKey, Track, WaitNext},
	skill::Skill,
	traits::get_ray::GetRayFromCamera,
};
use bevy::{
	ecs::system::EntityCommands,
	prelude::{Camera, Commands, Entity, GlobalTransform, Query, Ray},
	window::Window,
};

type Components<'a> = (
	Entity,
	&'a Schedule,
	&'a mut Queue,
	Option<&'a Track<Skill<Active>>>,
);

pub fn enqueue<TTools: GetRayFromCamera>(
	camera: Query<(&Camera, &GlobalTransform)>,
	window: Query<&Window>,
	mut agents: Query<Components>,
	mut commands: Commands,
) {
	if agents.is_empty() {
		return;
	}

	let (camera, camera_transform) = camera.single();
	let window = window.single();
	let ray = TTools::get_ray(camera, camera_transform, window);

	for (agent, schedule, mut queue, tracks) in &mut agents {
		enqueue_skills(agent, schedule, &mut queue, tracks, &mut commands, ray);
		commands.entity(agent).remove::<Schedule>();
	}
}

fn enqueue_skills(
	agent: Entity,
	schedule: &Schedule,
	queue: &mut Queue,
	track: Option<&Track<Skill<Active>>>,
	commands: &mut Commands,
	ray: Option<Ray>,
) {
	for slot in &schedule.skills {
		enqueue_skill(agent, schedule, queue, track, slot, commands, ray);
	}
}

fn enqueue_skill(
	agent: Entity,
	schedule: &Schedule,
	queue: &mut Queue,
	track: Option<&Track<Skill<Active>>>,
	slot: (&SlotKey, &Skill),
	commands: &mut Commands,
	ray: Option<Ray>,
) {
	let (slot, skill) = slot;
	let slot = *slot;
	let Some(new) = ray.map(|ray| skill.clone().with(Queued { ray, slot })) else {
		return;
	};

	if schedule.mode == ScheduleMode::Enqueue {
		return enqueue_to(queue, &new);
	}

	let Some(track) = track else {
		return override_hard(queue, &new, &mut commands.entity(agent));
	};

	if !track.value.soft_override || !new.soft_override {
		return override_hard(queue, &new, &mut commands.entity(agent));
	}

	override_soft(queue, &new);
}

fn enqueue_to(queue: &mut Queue, new: &Skill<Queued>) {
	queue.0.push_back(new.clone());
}

fn override_soft(queue: &mut Queue, new: &Skill<Queued>) {
	queue.0 = vec![new.clone()].into();
}

fn override_hard(queue: &mut Queue, new: &Skill<Queued>, agent: &mut EntityCommands) {
	queue.0 = vec![new.clone()].into();
	agent.insert(WaitNext);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Schedule, ScheduleMode, Side, WaitNext},
		skill::Cast,
	};
	use bevy::{
		prelude::{App, Camera, Camera3dBundle, GlobalTransform, Ray, Update, Vec3},
		utils::default,
		window::Window,
	};
	use mockall::mock;
	use std::time::Duration;

	const TEST_RAY: Ray = Ray {
		origin: Vec3::ONE,
		direction: Vec3::Z,
	};

	struct _ToolsSomeRay;

	impl GetRayFromCamera for _ToolsSomeRay {
		fn get_ray(
			_camera: &Camera,
			_camera_transform: &GlobalTransform,
			_window: &Window,
		) -> Option<Ray> {
			Some(TEST_RAY)
		}
	}

	macro_rules! setup_mock {
		($struct_name:ident) => {
			mock! {
				$struct_name {}
				impl GetRayFromCamera for $struct_name{
					fn get_ray(
						_camera: &Camera,
						_camera_transform: &GlobalTransform,
						_window: &Window,
					) -> Option<Ray> {}
				}
			}
		};
	}

	fn setup<TTools: GetRayFromCamera + 'static>() -> App {
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
		app.add_systems(Update, enqueue::<TTools>);

		app
	}

	#[test]
	fn set_enqueue() {
		let mut app = setup::<_ToolsSomeRay>();
		let agent = app
			.world
			.spawn((
				Schedule {
					mode: ScheduleMode::Enqueue,
					skills: [(
						SlotKey::Hand(Side::Off),
						Skill {
							cast: Cast {
								pre: Duration::from_millis(100),
								..default()
							},
							..default()
						},
					)]
					.into(),
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
						slot: SlotKey::Hand(Side::Off)
					},
					..default()
				},
			],
			queue.0.iter().collect::<Vec<&Skill<Queued>>>()
		);
	}

	#[test]
	fn set_override() {
		let mut app = setup::<_ToolsSomeRay>();
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
					skills: [(SlotKey::Hand(Side::Off), new_skill.clone())].into(),
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
					slot: SlotKey::Hand(Side::Off)
				})],
				true
			),
			(queue.0.iter().collect(), agent.contains::<WaitNext>())
		);
	}

	#[test]
	fn set_override_without_wait_next_when_new_and_running_soft_override() {
		let mut app = setup::<_ToolsSomeRay>();
		let running_skill = Skill {
			name: "running current",
			soft_override: true,
			..default()
		};
		let new_skill = Skill {
			name: "new",
			soft_override: true,
			..default()
		};

		let agent = app
			.world
			.spawn((
				Track::new(running_skill.clone()),
				Schedule {
					mode: ScheduleMode::Override,
					skills: [(SlotKey::Hand(Side::Off), new_skill.clone())].into(),
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
					slot: SlotKey::Hand(Side::Off)
				})],
				&Track::new(running_skill),
				false,
			),
			(
				queue.0.iter().collect::<Vec<&Skill<Queued>>>(),
				agent.get::<Track<Skill<Active>>>().unwrap(),
				agent.contains::<WaitNext>(),
			)
		);
	}

	#[test]
	fn set_override_with_wait_next_when_soft_override_running_soft_override_false() {
		let mut app = setup::<_ToolsSomeRay>();
		let running_skill = Skill {
			name: "running",
			soft_override: false,
			..default()
		};
		let new_skill = Skill {
			name: "new",
			soft_override: true,
			..default()
		};

		let agent = app
			.world
			.spawn((
				Track::new(running_skill.clone()),
				Schedule {
					mode: ScheduleMode::Override,
					skills: [(SlotKey::Hand(Side::Off), new_skill.clone())].into(),
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
					slot: SlotKey::Hand(Side::Off)
				})],
				&Track::new(running_skill),
				true,
			),
			(
				queue.0.iter().collect::<Vec<&Skill<Queued>>>(),
				agent.get::<Track<Skill<Active>>>().unwrap(),
				agent.contains::<WaitNext>(),
			)
		);
	}

	#[test]
	fn set_override_with_wait_next_when_soft_override_new_soft_override_false() {
		let mut app = setup::<_ToolsSomeRay>();
		let running_skill = Skill {
			name: "running",
			soft_override: true,
			..default()
		};
		let new_skill = Skill {
			name: "new",
			soft_override: false,
			..default()
		};

		let agent = app
			.world
			.spawn((
				Track::new(running_skill.clone()),
				Schedule {
					mode: ScheduleMode::Override,
					skills: [(SlotKey::Hand(Side::Off), new_skill.clone())].into(),
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
					slot: SlotKey::Hand(Side::Off)
				})],
				&Track::new(running_skill),
				true,
			),
			(
				queue.0.iter().collect::<Vec<&Skill<Queued>>>(),
				agent.get::<Track<Skill<Active>>>().unwrap(),
				agent.contains::<WaitNext>(),
			)
		);
	}

	#[test]
	fn remove_schedule() {
		let mut app = setup::<_ToolsSomeRay>();
		let schedule = Schedule {
			mode: ScheduleMode::Override,
			skills: [(SlotKey::Hand(Side::Off), Skill::default())].into(),
		};
		let agent = app.world.spawn((schedule, Queue([].into()))).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Schedule>());
	}

	setup_mock!(_RayFromCam);

	#[test]
	fn ray_from_camera_and_window() {
		let get_ray = Mock_RayFromCam::get_ray_context();
		get_ray
			.expect()
			.withf(|cam, cam_transform, window| {
				*cam_transform == GlobalTransform::from_xyz(4., 3., 2.)
				// using specific values for non-equatable variables
				&& cam.order == 42 && window.title == "Window"
			})
			.times(1)
			.return_const(TEST_RAY);

		let mut app = setup::<Mock_RayFromCam>();
		app.world.spawn((
			Schedule {
				mode: ScheduleMode::Override,
				skills: [(SlotKey::Hand(Side::Off), Skill::default())].into(),
			},
			Queue([].into()),
		));

		app.update();
	}

	setup_mock!(_GetRayFromCamNotCalled);

	#[test]
	fn do_not_produce_ray_when_nothing_scheduled() {
		let get_ray = Mock_GetRayFromCamNotCalled::get_ray_context();
		get_ray.expect().times(0).return_const(TEST_RAY);

		let mut app = setup::<Mock_GetRayFromCamNotCalled>();
		app.world.spawn(Queue([].into()));

		app.update();
	}

	#[test]
	fn try_soft_override_on_enqueue() {
		let mut app = setup::<_ToolsSomeRay>();
		app.world.spawn((
			Schedule {
				mode: ScheduleMode::Enqueue,
				skills: [(SlotKey::Hand(Side::Off), Skill::default())].into(),
			},
			Track::new(Skill::<Active>::default()),
			Queue([].into()),
		));

		app.update();
	}
}
