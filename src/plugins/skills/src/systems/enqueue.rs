use crate::{
	components::{Queue, Schedule, SideUnset, SlotKey, Track},
	skill::{Active, PlayerSkills, Queued, Skill},
	traits::WithComponent,
};
use bevy::{
	ecs::{
		system::{EntityCommands, Resource},
		world::Mut,
	},
	prelude::{Commands, Entity, Query},
	transform::components::GlobalTransform,
};
use common::components::Idle;

type Components<'a> = (
	Entity,
	&'a Schedule,
	&'a mut Queue,
	Option<&'a mut Track<Skill<PlayerSkills<SideUnset>, Active>>>,
);

pub(crate) fn enqueue<TTargetIds: WithComponent<GlobalTransform> + Resource>(
	mut agents: Query<Components>,
	mut commands: Commands,
) {
	if agents.is_empty() {
		return;
	}

	for (agent, schedule, queue, active) in &mut agents {
		let Some(mut agent) = commands.get_entity(agent) else {
			continue;
		};
		agent.remove::<Schedule>();
		apply_schedule(schedule, queue, active, agent);
	}
}

type ActiveSkill<'a> = Option<Mut<'a, Track<Skill<PlayerSkills<SideUnset>, Active>>>>;

fn apply_schedule(
	schedule: &Schedule,
	mut queue: Mut<Queue>,
	active: ActiveSkill,
	agent: EntityCommands,
) {
	match (schedule, active, queue.0.back_mut()) {
		(Schedule::Override(new), Some(active), ..) if both_soft(&active, new) => {
			override_soft(queue, as_queued(new.clone()));
		}
		(Schedule::Override(new), ..) => {
			override_hard(queue, as_queued(new.clone()), agent);
		}
		(Schedule::Enqueue(new), ..) => {
			enqueue_to(queue, as_queued(new.clone()));
		}
		(Schedule::StopAimAfter(time), .., Some(last_queued)) => {
			last_queued.cast.aim = *time;
		}
		(Schedule::StopAimAfter(time), Some(mut active), ..) => {
			active.value.cast.aim = *time;
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

fn as_queued((slot_key, skill): (SlotKey, Skill)) -> Skill<PlayerSkills<SideUnset>, Queued> {
	skill.with(Queued(slot_key))
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
	agent.try_insert(Idle);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{skill::Cast, traits::WithComponent};
	use bevy::{
		prelude::{App, Update},
		transform::components::GlobalTransform,
		utils::default,
	};
	use common::{
		components::{Outdated, Side},
		resources::ColliderInfo,
	};
	use std::{
		sync::{Arc, Mutex},
		time::Duration,
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

	fn setup() -> App {
		let mut app = App::new();

		app.add_systems(Update, enqueue::<_FakeTargetIds>);

		app
	}

	#[test]
	fn set_enqueue() {
		let mut app = setup();
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
					data: Queued(SlotKey::Hand(Side::Off)),
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
		let mut app = setup();
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
				vec![&new_skill.with(Queued(SlotKey::Hand(Side::Off)))],
				true
			),
			(queue.0.iter().collect(), agent.contains::<Idle>())
		);
	}

	#[test]
	fn set_override_without_wait_next_when_new_and_running_soft_override() {
		let mut app = setup();
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
				vec![&new_skill.with(Queued(SlotKey::Hand(Side::Off)))],
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
				agent.contains::<Idle>(),
			)
		);
	}

	#[test]
	fn set_override_with_wait_next_when_running_soft_override_false() {
		let mut app = setup();
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
				vec![&new_skill.with(Queued(SlotKey::Hand(Side::Off)))],
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
				agent.contains::<Idle>(),
			)
		);
	}

	#[test]
	fn set_override_with_wait_next_when_soft_override_new_soft_override_false() {
		let mut app = setup();
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
				vec![&new_skill.with(Queued(SlotKey::Hand(Side::Off)))],
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
				agent.contains::<Idle>(),
			)
		);
	}

	#[test]
	fn remove_schedule() {
		let mut app = setup();
		let schedule = Schedule::Override((SlotKey::Hand(Side::Off), Skill::default()));
		let agent = app.world.spawn((schedule, Queue::default())).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Schedule>());
	}

	#[test]
	fn try_soft_override_on_enqueue() {
		let mut app = setup();
		app.world.spawn((
			Schedule::Enqueue((SlotKey::Hand(Side::Off), Skill::default())),
			Track::new(Skill::<PlayerSkills<SideUnset>, Active>::default()),
			Queue::default(),
		));

		app.update();
	}

	#[test]
	fn update_aim_in_queue() {
		let mut app = setup();
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
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Schedule::StopAimAfter(Duration::from_millis(100)),
				Queue::<PlayerSkills<SideUnset>>([].into()),
				Track::new(Skill::<PlayerSkills<SideUnset>, Active> {
					name: "active skill",
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
		let mut app = setup();
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
				Track::new(Skill::<PlayerSkills<SideUnset>, Active> {
					name: "active skill",
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
}
