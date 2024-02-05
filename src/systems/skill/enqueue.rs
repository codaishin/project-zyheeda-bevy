use crate::{
	behaviors::meta::Target,
	components::{DequeueNext, PlayerSkills, Queue, Schedule, SideUnset, SlotKey, Track},
	resources::CamRay,
	skill::{Active, Queued, Skill},
	traits::with_component::WithComponent,
};
use bevy::{
	ecs::{
		system::{EntityCommands, Res, Resource},
		world::Mut,
	},
	prelude::{Commands, Entity, Query},
	transform::components::GlobalTransform,
};

type Components<'a> = (
	Entity,
	&'a Schedule,
	&'a mut Queue,
	Option<&'a mut Track<Skill<PlayerSkills<SideUnset>, Active>>>,
);

pub fn enqueue<TTargetIds: WithComponent<GlobalTransform> + Resource>(
	mut agents: Query<Components>,
	mut commands: Commands,
	transforms: Query<&GlobalTransform>,
	cam_ray: Res<CamRay>,
	target_ids: Res<TTargetIds>,
) {
	if agents.is_empty() {
		return;
	}

	let target = get_target(&cam_ray, target_ids.as_ref(), &transforms);

	for (agent, schedule, queue, active) in &mut agents {
		let mut agent = commands.entity(agent);
		agent.remove::<Schedule>();
		apply_schedule(schedule, queue, active, agent, &target);
	}
}

type ActiveSkill<'a> = Option<Mut<'a, Track<Skill<PlayerSkills<SideUnset>, Active>>>>;

fn apply_schedule(
	schedule: &Schedule,
	mut queue: Mut<Queue>,
	active: ActiveSkill,
	agent: EntityCommands,
	target: &Option<Target>,
) {
	let Some(target) = target else {
		return;
	};

	match (schedule, active, queue.0.back_mut()) {
		(Schedule::Override(new), Some(active), ..) if both_soft(&active, new) => {
			override_soft(queue, as_queued(new.clone(), target.clone()));
		}
		(Schedule::Override(new), ..) => {
			override_hard(queue, as_queued(new.clone(), target.clone()), agent);
		}
		(Schedule::Enqueue(new), ..) => {
			enqueue_to(queue, as_queued(new.clone(), target.clone()));
		}
		(Schedule::StopAimAfter(time), .., Some(last_queued)) => {
			last_queued.cast.aim = *time;
		}
		(Schedule::StopAimAfter(time), Some(mut active), ..) => {
			active.value.cast.aim = *time;
		}
		(Schedule::UpdateTarget, .., Some(last_queued)) => {
			last_queued.data.target = target.clone();
		}
		(Schedule::UpdateTarget, Some(mut active), ..) => {
			active.value.data.target = target.clone();
		}
		_ => {}
	}
}

fn both_soft(
	track: &Mut<Track<Skill<PlayerSkills<SideUnset>, Active>>>,
	(_, skill): &(SlotKey, Skill),
) -> bool {
	track.value.soft_override && skill.soft_override
}

fn get_target<TTargetIds: WithComponent<GlobalTransform>>(
	ray: &CamRay,
	target_ids: &TTargetIds,
	transforms: &Query<&GlobalTransform>,
) -> Option<Target> {
	Some(Target {
		ray: ray.0?,
		collision_info: target_ids.with_component(transforms),
	})
}

fn as_queued(
	(slot_key, skill): (SlotKey, Skill),
	target: Target,
) -> Skill<PlayerSkills<SideUnset>, Queued> {
	skill.with(&Queued { target, slot_key })
}

fn enqueue_to(mut queue: Mut<Queue>, new: Skill<PlayerSkills<SideUnset>, Queued>) {
	queue.0.push_back(new);
}

fn override_soft(mut queue: Mut<Queue>, new: Skill<PlayerSkills<SideUnset>, Queued>) {
	queue.0 = vec![new].into();
}

fn override_hard(
	mut queue: Mut<Queue>,
	new: Skill<PlayerSkills<SideUnset>, Queued>,
	mut agent: EntityCommands,
) {
	queue.0 = vec![new].into();
	agent.insert(DequeueNext);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		behaviors::meta::Outdated,
		components::{DequeueNext, Schedule, Side},
		resources::ColliderInfo,
		skill::Cast,
	};
	use bevy::{
		prelude::{App, Ray, Update, Vec3},
		transform::components::GlobalTransform,
		utils::default,
	};
	use std::{
		sync::{Arc, Mutex},
		time::Duration,
	};

	const TEST_RAY: Ray = Ray {
		origin: Vec3::ONE,
		direction: Vec3::Z,
	};

	#[derive(Resource)]
	struct _FakeTargetIds {
		pub returns: Option<ColliderInfo<Outdated<GlobalTransform>>>,
		pub tracked_transform_args: Arc<Mutex<Vec<GlobalTransform>>>,
	}

	impl WithComponent<GlobalTransform> for _FakeTargetIds {
		fn with_component(
			&self,
			query: &Query<&GlobalTransform>,
		) -> Option<ColliderInfo<Outdated<GlobalTransform>>> {
			if let Ok(mut t) = self.tracked_transform_args.lock() {
				for transform in query {
					t.push(*transform);
				}
			}
			self.returns.clone()
		}
	}

	type FakeTargetTransforms = Option<ColliderInfo<Outdated<GlobalTransform>>>;
	type TrackedTransformArgs = Arc<Mutex<Vec<GlobalTransform>>>;

	fn setup(ray: Option<Ray>) -> (App, FakeTargetTransforms, TrackedTransformArgs) {
		let mut app = App::new();
		let tracked_transform_args = Arc::new(Mutex::new(vec![]));
		let fake_target_transforms = Some(ColliderInfo {
			collider: Outdated {
				entity: Entity::from_raw(42),
				component: GlobalTransform::from_xyz(1., 2., 3.),
			},
			root: Some(Outdated {
				entity: Entity::from_raw(43),
				component: GlobalTransform::from_xyz(4., 5., 6.),
			}),
		});

		app.insert_resource(CamRay(ray));
		app.insert_resource(_FakeTargetIds {
			returns: fake_target_transforms.clone(),
			tracked_transform_args: tracked_transform_args.clone(),
		});
		app.add_systems(Update, enqueue::<_FakeTargetIds>);

		(app, fake_target_transforms, tracked_transform_args)
	}

	#[test]
	fn set_enqueue() {
		let (mut app, collision_info, ..) = setup(Some(TEST_RAY));
		let agent = app
			.world
			.spawn((
				Schedule::Enqueue((
					SlotKey::Hand(Side::Off),
					Skill {
						cast: Cast {
							pre: Duration::from_millis(100),
							..default()
						},
						..default()
					},
				)),
				Queue::<PlayerSkills<SideUnset>>(
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
						target: Target {
							ray: TEST_RAY,
							collision_info,
						},
						slot_key: SlotKey::Hand(Side::Off),
					},
					..default()
				},
			],
			queue
				.0
				.iter()
				.collect::<Vec<&Skill<PlayerSkills<SideUnset>, Queued>>>()
		);
	}

	#[test]
	fn set_override() {
		let (mut app, collision_info, ..) = setup(Some(TEST_RAY));
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
				Schedule::Override((SlotKey::Hand(Side::Off), new_skill.clone())),
				Queue::<PlayerSkills<SideUnset>>(
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
				vec![&new_skill.with(&Queued {
					target: Target {
						ray: TEST_RAY,
						collision_info,
					},
					slot_key: SlotKey::Hand(Side::Off),
				})],
				true
			),
			(queue.0.iter().collect(), agent.contains::<DequeueNext>())
		);
	}

	#[test]
	fn call_with_correct_transform_query() {
		let (mut app, .., tracked_transform_args) = setup(Some(TEST_RAY));
		let transforms = vec![
			GlobalTransform::from_xyz(11., 12., 13.),
			GlobalTransform::from_xyz(1., 11., 111.),
		];
		for transform in &transforms {
			app.world.spawn(*transform);
		}
		app.world.spawn((
			Schedule::Enqueue((
				SlotKey::Hand(Side::Off),
				Skill {
					cast: Cast {
						pre: Duration::from_millis(100),
						..default()
					},
					..default()
				},
			)),
			Queue::<PlayerSkills<SideUnset>>(
				[Skill {
					cast: Cast {
						pre: Duration::from_millis(1),
						..default()
					},
					..default()
				}]
				.into(),
			),
		));

		app.update();

		let Ok(args) = tracked_transform_args.try_lock() else {
			panic!("Failed to read tracked arguments");
		};

		let args: Vec<_> = args.iter().cloned().collect();

		assert_eq!(transforms, args);
	}

	#[test]
	fn set_override_without_wait_next_when_new_and_running_soft_override() {
		let (mut app, collision_info, ..) = setup(Some(TEST_RAY));
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
				Schedule::Override((SlotKey::Hand(Side::Off), new_skill.clone())),
				Queue::default(),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(
				vec![&new_skill.with(&Queued {
					target: Target {
						ray: TEST_RAY,
						collision_info,
					},
					slot_key: SlotKey::Hand(Side::Off),
				})],
				&Track::new(running_skill),
				false,
			),
			(
				queue
					.0
					.iter()
					.collect::<Vec<&Skill<PlayerSkills<SideUnset>, Queued>>>(),
				agent
					.get::<Track<Skill<PlayerSkills<SideUnset>, Active>>>()
					.unwrap(),
				agent.contains::<DequeueNext>(),
			)
		);
	}

	#[test]
	fn set_override_with_wait_next_when_running_soft_override_false() {
		let (mut app, collision_info, ..) = setup(Some(TEST_RAY));
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
				Schedule::Override((SlotKey::Hand(Side::Off), new_skill.clone())),
				Queue::default(),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(
				vec![&new_skill.with(&Queued {
					target: Target {
						ray: TEST_RAY,
						collision_info,
					},
					slot_key: SlotKey::Hand(Side::Off),
				})],
				&Track::new(running_skill),
				true,
			),
			(
				queue
					.0
					.iter()
					.collect::<Vec<&Skill<PlayerSkills<SideUnset>, Queued>>>(),
				agent
					.get::<Track<Skill<PlayerSkills<SideUnset>, Active>>>()
					.unwrap(),
				agent.contains::<DequeueNext>(),
			)
		);
	}

	#[test]
	fn set_override_with_wait_next_when_soft_override_new_soft_override_false() {
		let (mut app, collision_info, ..) = setup(Some(TEST_RAY));
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
				Schedule::Override((SlotKey::Hand(Side::Off), new_skill.clone())),
				Queue::default(),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(
				vec![&new_skill.with(&Queued {
					target: Target {
						ray: TEST_RAY,
						collision_info,
					},
					slot_key: SlotKey::Hand(Side::Off),
				})],
				&Track::new(running_skill),
				true,
			),
			(
				queue
					.0
					.iter()
					.collect::<Vec<&Skill<PlayerSkills<SideUnset>, Queued>>>(),
				agent
					.get::<Track<Skill<PlayerSkills<SideUnset>, Active>>>()
					.unwrap(),
				agent.contains::<DequeueNext>(),
			)
		);
	}

	#[test]
	fn remove_schedule() {
		let (mut app, ..) = setup(Some(TEST_RAY));
		let schedule = Schedule::Override((SlotKey::Hand(Side::Off), Skill::default()));
		let agent = app.world.spawn((schedule, Queue::default())).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Schedule>());
	}

	#[test]
	fn try_soft_override_on_enqueue() {
		let (mut app, ..) = setup(Some(TEST_RAY));
		app.world.spawn((
			Schedule::Enqueue((SlotKey::Hand(Side::Off), Skill::default())),
			Track::new(Skill::<PlayerSkills<SideUnset>, Active>::default()),
			Queue::default(),
		));

		app.update();
	}

	#[test]
	fn update_aim_in_queue() {
		let (mut app, ..) = setup(Some(TEST_RAY));
		let agent = app
			.world
			.spawn((
				Schedule::StopAimAfter(Duration::from_secs(3)),
				Queue::<PlayerSkills<SideUnset>>(
					[
						Skill { ..default() },
						Skill {
							name: "last in queue",
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
				&Skill { ..default() },
				&Skill {
					name: "last in queue",
					cast: Cast {
						aim: Duration::from_secs(3),
						..default()
					},
					..default()
				},
			],
			queue
				.0
				.iter()
				.collect::<Vec<&Skill<PlayerSkills<SideUnset>, Queued>>>()
		);
	}

	#[test]
	fn update_aim_active() {
		let (mut app, ..) = setup(Some(TEST_RAY));
		let agent = app
			.world
			.spawn((
				Schedule::StopAimAfter(Duration::from_millis(100)),
				Queue::<PlayerSkills<SideUnset>>([].into()),
				Track::new(Skill {
					name: "active skill",
					data: Active::default(),
					..default()
				}),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let active = agent
			.get::<Track<Skill<PlayerSkills<SideUnset>, Active>>>()
			.unwrap();

		assert_eq!(
			Skill {
				name: "active skill",
				data: Active::default(),
				cast: Cast {
					aim: Duration::from_millis(100),
					..default()
				},
				..default()
			},
			active.value
		);
	}

	#[test]
	fn aim_last_in_queue_even_with_active() {
		let (mut app, ..) = setup(Some(TEST_RAY));
		let agent = app
			.world
			.spawn((
				Schedule::StopAimAfter(Duration::from_millis(101)),
				Queue::<PlayerSkills<SideUnset>>(
					[
						Skill { ..default() },
						Skill {
							name: "last in queue",
							..default()
						},
					]
					.into(),
				),
				Track::new(Skill {
					name: "active skill",
					data: Active::default(),
					..default()
				}),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let active = agent
			.get::<Track<Skill<PlayerSkills<SideUnset>, Active>>>()
			.unwrap();
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(
				Skill {
					name: "active skill",
					data: Active { ..default() },
					..default()
				},
				vec![
					&Skill { ..default() },
					&Skill {
						name: "last in queue",
						data: Queued::default(),
						cast: Cast {
							aim: Duration::from_millis(101),
							..default()
						},
						..default()
					},
				],
			),
			(active.value.clone(), queue.0.iter().collect::<Vec<_>>())
		);
	}

	#[test]
	fn update_target_of_active() {
		let (mut app, collision_info, ..) = setup(Some(TEST_RAY));
		let agent = app
			.world
			.spawn((
				Schedule::UpdateTarget,
				Queue::<PlayerSkills<SideUnset>>([].into()),
				Track::new(Skill {
					name: "active skill",
					data: Active {
						target: Target::default(),
						..default()
					},
					..default()
				}),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let track = agent
			.get::<Track<Skill<PlayerSkills<SideUnset>, Active>>>()
			.unwrap();

		assert_eq!(
			Target {
				ray: TEST_RAY,
				collision_info,
			},
			track.value.data.target
		);
	}

	#[test]
	fn update_target_of_last_queued() {
		let (mut app, collision_info, ..) = setup(Some(TEST_RAY));
		let agent = app
			.world
			.spawn((
				Schedule::UpdateTarget,
				Queue::<PlayerSkills<SideUnset>>(
					[
						Skill {
							name: "not last",
							..default()
						},
						Skill {
							name: "last",
							..default()
						},
					]
					.into(),
				),
				Track::new(Skill {
					data: Active::default(),
					..default()
				}),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let active = agent
			.get::<Track<Skill<PlayerSkills<SideUnset>, Active>>>()
			.unwrap();
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(
				&Track::new(Skill {
					data: Active::default(),
					..default()
				}),
				vec![
					&Skill {
						name: "not last",
						..default()
					},
					&Skill {
						name: "last",
						data: Queued {
							target: Target {
								ray: TEST_RAY,
								collision_info,
							},
							..default()
						},
						..default()
					},
				]
			),
			(active, queue.0.iter().collect::<Vec<_>>()),
		);
	}
}
