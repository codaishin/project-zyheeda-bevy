use crate::components::{Queue, Track};
use bevy::prelude::{Commands, Entity, Query, With};
use common::components::Idle;

pub(crate) fn dequeue(mut commands: Commands, mut agents: Query<(Entity, &mut Queue), With<Idle>>) {
	for (agent, mut queue) in agents.iter_mut() {
		let Some(mut agent) = commands.get_entity(agent) else {
			continue;
		};

		let Some(skill) = queue.0.pop_front() else {
			continue;
		};

		agent.try_insert(Track::new(skill.to_active()));
		agent.remove::<Idle>();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::SlotKey,
		skill::{Active, Cast, Queued, Skill},
	};
	use bevy::prelude::{default, App, Update};
	use std::time::Duration;

	#[test]
	fn pop_first_behavior_to_agent() {
		let mut app = App::new();
		let queue = Queue(
			[(Skill {
				cast: Cast {
					pre: Duration::from_millis(42),
					..default()
				},
				data: Queued(SlotKey::SkillSpawn),
				..default()
			})]
			.into(),
		);
		let agent = app.world.spawn((queue, Idle)).id();

		app.add_systems(Update, dequeue);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(Some(Active(SlotKey::SkillSpawn)), false, 0),
			(
				agent
					.get::<Track<Skill<Active>>>()
					.map(|t| t.value.data.clone()),
				agent.contains::<Idle>(),
				queue.0.len()
			)
		);
	}

	#[test]
	fn do_not_pop_when_not_idle() {
		let mut app = App::new();
		let queue = Queue([Skill::default()].into());
		let agent = app.world.spawn(queue).id();

		app.add_systems(Update, dequeue);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(false, 1),
			(agent.contains::<Track<Skill<Queued>>>(), queue.0.len())
		);
	}
}
