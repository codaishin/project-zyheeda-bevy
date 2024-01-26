use crate::{
	components::{
		PlayerSkills,
		Queue,
		Schedule,
		ScheduleMode,
		SideUnset,
		SlotKey,
		Track,
		WaitNext,
	},
	resources::CamRay,
	skill::{Active, Queued, SelectInfo, Skill},
};
use bevy::{
	ecs::system::{EntityCommands, Res},
	prelude::{Commands, Entity, Query, Ray},
	utils::default,
};

type Components<'a> = (
	Entity,
	&'a Schedule,
	&'a mut Queue,
	Option<&'a Track<Skill<PlayerSkills<SideUnset>, Active>>>,
);

pub fn enqueue(mut agents: Query<Components>, mut commands: Commands, cam_ray: Res<CamRay>) {
	if agents.is_empty() {
		return;
	}

	let ray = cam_ray.0;

	for (agent, schedule, mut queue, tracks) in &mut agents {
		enqueue_skills(agent, schedule, &mut queue, tracks, &mut commands, ray);
		commands.entity(agent).remove::<Schedule>();
	}
}

fn enqueue_skills(
	agent: Entity,
	schedule: &Schedule,
	queue: &mut Queue,
	track: Option<&Track<Skill<PlayerSkills<SideUnset>, Active>>>,
	commands: &mut Commands,
	ray: Option<Ray>,
) {
	for scheduled in &schedule.skills {
		enqueue_skill(agent, schedule, queue, track, scheduled, commands, ray);
	}
}

fn enqueue_skill(
	agent: Entity,
	schedule: &Schedule,
	queue: &mut Queue,
	track: Option<&Track<Skill<PlayerSkills<SideUnset>, Active>>>,
	(slot, skill): (&SlotKey, &Skill),
	commands: &mut Commands,
	ray: Option<Ray>,
) {
	let slot = *slot;
	let Some(new) = ray.map(|ray| {
		skill.clone().with(&Queued {
			select_info: SelectInfo { ray, ..default() },
			slot_key: slot,
		})
	}) else {
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

fn enqueue_to(queue: &mut Queue, new: &Skill<PlayerSkills<SideUnset>, Queued>) {
	queue.0.push_back(new.clone());
}

fn override_soft(queue: &mut Queue, new: &Skill<PlayerSkills<SideUnset>, Queued>) {
	queue.0 = vec![new.clone()].into();
}

fn override_hard(
	queue: &mut Queue,
	new: &Skill<PlayerSkills<SideUnset>, Queued>,
	agent: &mut EntityCommands,
) {
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
		prelude::{App, Ray, Update, Vec3},
		utils::default,
	};
	use std::time::Duration;

	const TEST_RAY: Ray = Ray {
		origin: Vec3::ONE,
		direction: Vec3::Z,
	};

	fn setup(ray: Option<Ray>) -> App {
		let mut app = App::new();

		app.insert_resource(CamRay(ray));
		app.add_systems(Update, enqueue);

		app
	}

	#[test]
	fn set_enqueue() {
		let mut app = setup(Some(TEST_RAY));
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
						select_info: SelectInfo {
							ray: TEST_RAY,
							..default()
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
		let mut app = setup(Some(TEST_RAY));
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
					select_info: SelectInfo {
						ray: TEST_RAY,
						..default()
					},
					slot_key: SlotKey::Hand(Side::Off),
				})],
				true
			),
			(queue.0.iter().collect(), agent.contains::<WaitNext>())
		);
	}

	#[test]
	fn set_override_without_wait_next_when_new_and_running_soft_override() {
		let mut app = setup(Some(TEST_RAY));
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
				Queue::default(),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(
				vec![&new_skill.with(&Queued {
					select_info: SelectInfo {
						ray: TEST_RAY,
						..default()
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
				agent.contains::<WaitNext>(),
			)
		);
	}

	#[test]
	fn set_override_with_wait_next_when_soft_override_running_soft_override_false() {
		let mut app = setup(Some(TEST_RAY));
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
				Queue::default(),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(
				vec![&new_skill.with(&Queued {
					select_info: SelectInfo {
						ray: TEST_RAY,
						..default()
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
				agent.contains::<WaitNext>(),
			)
		);
	}

	#[test]
	fn set_override_with_wait_next_when_soft_override_new_soft_override_false() {
		let mut app = setup(Some(TEST_RAY));
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
				Queue::default(),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(
				vec![&new_skill.with(&Queued {
					select_info: SelectInfo {
						ray: TEST_RAY,
						..default()
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
				agent.contains::<WaitNext>(),
			)
		);
	}

	#[test]
	fn remove_schedule() {
		let mut app = setup(Some(TEST_RAY));
		let schedule = Schedule {
			mode: ScheduleMode::Override,
			skills: [(SlotKey::Hand(Side::Off), Skill::default())].into(),
		};
		let agent = app.world.spawn((schedule, Queue::default())).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Schedule>());
	}

	#[test]
	fn try_soft_override_on_enqueue() {
		let mut app = setup(Some(TEST_RAY));
		app.world.spawn((
			Schedule {
				mode: ScheduleMode::Enqueue,
				skills: [(SlotKey::Hand(Side::Off), Skill::default())].into(),
			},
			Track::new(Skill::<PlayerSkills<SideUnset>, Active>::default()),
			Queue::default(),
		));

		app.update();
	}
}
