use crate::{
	components::{Active, Queue, Queued, Schedule, ScheduleMode, Skill, SlotKey, WaitNext},
	traits::{get_ray::GetRayFromCamera, try_soft_override::TrySoftOverride},
};
use bevy::{
	prelude::{Camera, Commands, Entity, GlobalTransform, Mut, Query, Ray},
	window::Window,
};

pub fn enqueue<TTools: GetRayFromCamera + TrySoftOverride>(
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
	let ray = TTools::get_ray(camera, camera_transform, window);

	for (agent, schedule, mut queue, mut running) in &mut agents {
		enqueue_skills::<TTools>(
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

fn enqueue_skills<TTools: TrySoftOverride>(
	agent: Entity,
	schedule: &Schedule,
	queue: &mut Queue,
	running: &mut Option<Mut<Skill<Active>>>,
	commands: &mut Commands,
	ray: Option<Ray>,
) {
	for slot in &schedule.skills {
		enqueue_skill::<TTools>(agent, schedule, queue, running, slot, commands, ray);
	}
}

fn enqueue_skill<TTools: TrySoftOverride>(
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

	if schedule.mode == ScheduleMode::Enqueue {
		queue.0.push_back(new);
		return;
	}

	let override_soft = running
		.as_mut()
		.map(|running| TTools::try_soft_override(running, &mut new))
		.unwrap_or(false);

	queue.0 = vec![new].into();

	if !override_soft {
		commands.entity(agent).insert(WaitNext);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Cast, Schedule, ScheduleMode, Side, WaitNext};
	use bevy::{
		prelude::{App, Camera, Camera3dBundle, GlobalTransform, Ray, Update, Vec3},
		utils::default,
		window::Window,
	};
	use mockall::{mock, predicate::eq};
	use std::time::Duration;

	const TEST_RAY: Ray = Ray {
		origin: Vec3::ONE,
		direction: Vec3::Z,
	};

	struct _Tools;

	impl GetRayFromCamera for _Tools {
		fn get_ray(
			_camera: &Camera,
			_camera_transform: &GlobalTransform,
			_window: &Window,
		) -> Option<Ray> {
			Some(TEST_RAY)
		}
	}

	impl TrySoftOverride for _Tools {
		fn try_soft_override(_running: &mut Skill<Active>, _new: &mut Skill<Queued>) -> bool {
			false
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
				impl TrySoftOverride for $struct_name {
					fn try_soft_override(_running: &mut Skill<Active>, _new: &mut Skill<Queued>) -> bool {}
				}
			}
		};
	}

	fn setup<TTools: GetRayFromCamera + TrySoftOverride + 'static>() -> App {
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
		let mut app = setup::<_Tools>();
		let agent = app
			.world
			.spawn((
				Schedule {
					mode: ScheduleMode::Enqueue,
					skills: [(
						SlotKey::Hand(Side::Left),
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
		let mut app = setup::<_Tools>();
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

	setup_mock!(_A);

	#[test]
	fn set_override_without_wait_next_when_try_chain_true() {
		let mut app = setup::<Mock_A>();
		let running_skill = Skill {
			name: "running",
			..default()
		};
		let new_skill = Skill {
			name: "new",
			..default()
		};
		let get_ray = Mock_A::get_ray_context();
		let try_soft_override = Mock_A::try_soft_override_context();

		get_ray.expect().return_const(Some(TEST_RAY));
		try_soft_override
			.expect()
			.times(1)
			.with(
				eq(running_skill),
				eq(new_skill.with(Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Left),
				})),
			)
			.return_const(true);

		let agent = app
			.world
			.spawn((
				running_skill,
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
		let mut app = setup::<_Tools>();
		let schedule = Schedule {
			mode: ScheduleMode::Override,
			skills: [(SlotKey::Hand(Side::Left), Skill::default())].into(),
		};
		let agent = app.world.spawn((schedule, Queue([].into()))).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Schedule>());
	}

	setup_mock!(_B);

	#[test]
	fn ray_from_camera_and_window() {
		let get_ray = Mock_B::get_ray_context();
		let try_soft_override = Mock_B::try_soft_override_context();
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
		try_soft_override.expect().return_const(false);

		let mut app = setup::<Mock_B>();
		app.world.spawn((
			Schedule {
				mode: ScheduleMode::Override,
				skills: [(SlotKey::Hand(Side::Left), Skill::default())].into(),
			},
			Queue([].into()),
		));

		app.update();
	}

	setup_mock!(_C);

	#[test]
	fn do_not_produce_ray_when_nothing_scheduled() {
		let get_ray = Mock_C::get_ray_context();
		let try_soft_override = Mock_C::try_soft_override_context();
		let ray = Ray {
			origin: Vec3::ZERO,
			direction: Vec3::ONE,
		};

		get_ray.expect().times(0).return_const(ray);
		try_soft_override.expect().return_const(false);

		let mut app = setup::<Mock_C>();
		app.world.spawn(Queue([].into()));

		app.update();
	}

	setup_mock!(_D);

	#[test]
	fn do_try_chain_on_enqueue() {
		let get_ray = Mock_D::get_ray_context();
		let try_soft_override = Mock_D::try_soft_override_context();
		let ray = Ray {
			origin: Vec3::ZERO,
			direction: Vec3::ONE,
		};

		get_ray.expect().return_const(ray);
		try_soft_override.expect().never().return_const(false);

		let mut app = setup::<Mock_D>();
		app.world.spawn((
			Schedule {
				mode: ScheduleMode::Enqueue,
				skills: [(SlotKey::Hand(Side::Left), Skill::default())].into(),
			},
			Skill::<Active>::default(),
			Queue([].into()),
		));

		app.update();
	}
}
