use crate::{
	components::{Marker, Queue, WaitNext},
	markers::Idle,
};
use bevy::prelude::{Commands, Entity, Query, With};

pub fn dequeue(mut commands: Commands, mut agents: Query<(Entity, &mut Queue), With<WaitNext>>) {
	for (agent, mut queue) in agents.iter_mut() {
		let mut agent = commands.entity(agent);

		if let Some(skill) = queue.0.pop_front() {
			agent.insert(skill);
			agent.remove::<WaitNext>();
			agent.remove::<Marker<Idle>>();
		} else {
			agent.insert(Marker::<Idle>::new());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Cast, Queued, Skill, SlotKey, WaitNext};
	use bevy::prelude::{default, App, Ray, Update, Vec3};
	use std::time::Duration;

	const TEST_RAY: Ray = Ray {
		origin: Vec3::ONE,
		direction: Vec3::Y,
	};

	#[test]
	fn pop_first_behavior_to_agent() {
		let mut app = App::new();
		let queue = Queue(
			[(Skill {
				cast: Cast {
					pre: Duration::from_millis(42),
					..default()
				},
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::SkillSpawn,
				},
				..default()
			})]
			.into(),
		);
		let agent = app.world.spawn((queue, WaitNext)).id();

		app.add_systems(Update, dequeue);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(
				Some(Queued {
					ray: TEST_RAY,
					slot: SlotKey::SkillSpawn,
				}),
				false,
				0
			),
			(
				agent.get::<Skill<Queued>>().map(|s| s.data),
				agent.contains::<WaitNext>(),
				queue.0.len()
			)
		);
	}

	#[test]
	fn do_not_pop_when_not_waiting_next() {
		let mut app = App::new();
		let queue = Queue([Skill::default()].into());
		let agent = app.world.spawn(queue).id();

		app.add_systems(Update, dequeue);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(false, 1),
			(agent.contains::<Skill<Queued>>(), queue.0.len())
		);
	}

	#[test]
	fn idle_when_nothing_to_pop() {
		let mut app = App::new();
		let agent = app.world.spawn((Queue(default()), WaitNext)).id();

		app.add_systems(Update, dequeue);
		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Marker<Idle>>());
	}

	#[test]
	fn remove_idle_when_something_to_pop() {
		let mut app = App::new();
		let queue = Queue([Skill::default()].into());
		let idle = Marker::<Idle>::new();

		let agent = app.world.spawn((queue, WaitNext, idle)).id();
		app.add_systems(Update, dequeue);
		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Marker<Idle>>());
	}
}
