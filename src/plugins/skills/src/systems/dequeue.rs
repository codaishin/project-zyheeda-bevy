use crate::{
	components::Track,
	skill::{Active, Queued, Skill},
	traits::Dequeue,
};
use bevy::{
	ecs::{component::Component, query::Without},
	prelude::{Commands, Entity, Query},
};

pub(crate) fn dequeue<TDequeue: Dequeue<Skill<Queued>> + Component>(
	mut commands: Commands,
	mut agents: Query<(Entity, &mut TDequeue), Without<Track<Skill<Active>>>>,
) {
	for (agent, mut queue) in agents.iter_mut() {
		let Some(skill) = queue.dequeue() else {
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
	use mockall::automock;
	use std::time::Duration;

	#[derive(Component, Default)]
	struct _Queue {
		mock: Mock_Queue,
	}

	#[automock]
	impl Dequeue<Skill<Queued>> for _Queue {
		fn dequeue(&mut self) -> Option<Skill<Queued>> {
			self.mock.dequeue()
		}
	}

	#[test]
	fn dequeue_behavior_to_agent() {
		let mut app = App::new();
		let mut queue = _Queue::default();
		queue.mock.expect_dequeue().return_const(Some(Skill {
			cast: Cast {
				pre: Duration::from_millis(42),
				..default()
			},
			data: Queued(SlotKey::SkillSpawn),
			..default()
		}));
		let agent = app.world.spawn(queue).id();

		app.add_systems(Update, dequeue::<_Queue>);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(Active(SlotKey::SkillSpawn)),
			agent
				.get::<Track<Skill<Active>>>()
				.map(|t| t.value.data.clone()),
		);
	}

	#[test]
	fn dequeue_when_track_present() {
		let mut app = App::new();
		let mut queue = _Queue::default();
		queue.mock.expect_dequeue().never().return_const(None);

		app.add_systems(Update, dequeue::<_Queue>);
		app.update();
	}
}
