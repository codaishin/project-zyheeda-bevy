use crate::{
	components::{Idle, Queue},
	traits::{insert_into_entity::InsertIntoEntity, remove_from_entity::RemoveFromEntity},
};
use bevy::prelude::{Commands, Component, Entity, Query, With};

#[allow(clippy::type_complexity)]
pub fn dequeue<
	TAgent: Component,
	TBehavior: Copy + Send + Sync + InsertIntoEntity + RemoveFromEntity + 'static,
>(
	mut commands: Commands,
	mut agents: Query<(Entity, &mut Queue<TBehavior>), (With<TAgent>, With<Idle<TBehavior>>)>,
) {
	for (agent, mut queue) in agents.iter_mut() {
		let mut agent = commands.entity(agent);

		if let Some(behavior) = queue.0.pop_front() {
			behavior.insert_into_entity(&mut agent);
			agent.remove::<Idle<TBehavior>>();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Idle;
	use bevy::{
		ecs::system::EntityCommands,
		prelude::{App, Update},
	};

	#[derive(Clone, Copy)]
	enum Behavior {
		Sing,
	}

	#[derive(Component, Debug)]
	struct Sing;

	impl InsertIntoEntity for Behavior {
		fn insert_into_entity(self, entity: &mut EntityCommands) {
			entity.insert(Sing);
		}
	}

	impl RemoveFromEntity for Behavior {
		fn remove_from_entity(&self, entity: &mut EntityCommands) {
			entity.remove::<Sing>();
		}
	}

	#[derive(Component)]
	struct Agent;

	#[test]
	fn pop_first_behavior_to_agent() {
		let mut app = App::new();
		let queue = Queue([Behavior::Sing].into());
		let agent = Agent;
		let idle = Idle::<Behavior>::new();

		let agent = app.world.spawn((agent, queue, idle)).id();
		app.add_systems(Update, dequeue::<Agent, Behavior>);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<Behavior>>().unwrap();

		assert_eq!(
			(true, false, 0),
			(
				agent.contains::<Sing>(),
				agent.contains::<Idle<Behavior>>(),
				queue.0.len()
			)
		);
	}

	#[test]
	fn do_not_pop_when_not_idling() {
		let mut app = App::new();
		let queue = Queue([Behavior::Sing].into());
		let agent = Agent;

		let agent = app.world.spawn((agent, queue)).id();
		app.add_systems(Update, dequeue::<Agent, Behavior>);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<Behavior>>().unwrap();

		assert_eq!((false, 1), (agent.contains::<Sing>(), queue.0.len()));
	}
}
