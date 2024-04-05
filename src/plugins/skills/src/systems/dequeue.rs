use crate::{
	components::{Queue, Track},
	skill::{Active, Skill},
};
use bevy::{
	ecs::query::Without,
	prelude::{Commands, Entity, Query},
};

pub(crate) fn dequeue(
	mut commands: Commands,
	mut agents: Query<(Entity, &mut Queue), Without<Track<Skill<Active>>>>,
) {
	for (agent, mut queue) in agents.iter_mut() {
		let Some(skill) = queue.0.pop_front() else {
			continue;
		};

		let Some(mut agent) = commands.get_entity(agent) else {
			continue;
		};

		agent.try_insert(Track::new(skill.to_active()));
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
	use common::components::Side;
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
		let agent = app.world.spawn(queue).id();

		app.add_systems(Update, dequeue);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(Some(Active(SlotKey::SkillSpawn)), 0),
			(
				agent
					.get::<Track<Skill<Active>>>()
					.map(|t| t.value.data.clone()),
				queue.0.len()
			)
		);
	}

	#[test]
	fn do_not_pop_when_track_present() {
		let mut app = App::new();
		let queue = Queue([Skill::default()].into());
		let agent = app
			.world
			.spawn((
				queue,
				Track::new(Skill {
					data: Active(SlotKey::Hand(Side::Main)),
					..default()
				}),
			))
			.id();

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
