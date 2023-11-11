use crate::{
	components::{
		marker::{Idle, Marker},
		Queue,
		WaitNext,
	},
	traits::insert_into_entity::InsertIntoEntity,
};
use bevy::prelude::{Commands, Entity, Query, With};

#[allow(clippy::type_complexity)]
pub fn dequeue<TBehavior: Copy + Send + Sync + InsertIntoEntity + 'static>(
	mut commands: Commands,
	mut agents: Query<(Entity, &mut Queue<TBehavior>), With<WaitNext<TBehavior>>>,
) {
	for (agent, mut queue) in agents.iter_mut() {
		let mut agent = commands.entity(agent);

		if let Some(behavior) = queue.0.pop_front() {
			behavior.insert_into_entity(&mut agent);
			agent.remove::<WaitNext<TBehavior>>();
			agent.remove::<Marker<Idle>>();
		} else {
			agent.insert(Marker::<Idle>::new());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::WaitNext;
	use bevy::{
		ecs::system::EntityCommands,
		prelude::{default, App, Component, Update},
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

	#[test]
	fn pop_first_behavior_to_agent() {
		let mut app = App::new();
		let queue = Queue([Behavior::Sing].into());
		let wait = WaitNext::<Behavior>::new();

		let agent = app.world.spawn((queue, wait)).id();
		app.add_systems(Update, dequeue::<Behavior>);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<Behavior>>().unwrap();

		assert_eq!(
			(true, false, 0),
			(
				agent.contains::<Sing>(),
				agent.contains::<WaitNext<Behavior>>(),
				queue.0.len()
			)
		);
	}

	#[test]
	fn do_not_pop_when_not_waiting_next() {
		let mut app = App::new();
		let queue = Queue([Behavior::Sing].into());

		let agent = app.world.spawn(queue).id();
		app.add_systems(Update, dequeue::<Behavior>);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<Behavior>>().unwrap();

		assert_eq!((false, 1), (agent.contains::<Sing>(), queue.0.len()));
	}

	#[test]
	fn idle_when_nothing_to_pop() {
		let mut app = App::new();
		let queue: Queue<Behavior> = Queue(default());
		let wait = WaitNext::<Behavior>::new();

		let agent = app.world.spawn((queue, wait)).id();
		app.add_systems(Update, dequeue::<Behavior>);
		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Marker<Idle>>());
	}

	#[test]
	fn remove_idle_when_something_to_pop() {
		let mut app = App::new();
		let queue = Queue([Behavior::Sing].into());
		let wait = WaitNext::<Behavior>::new();
		let idle = Marker::<Idle>::new();

		let agent = app.world.spawn((queue, wait, idle)).id();
		app.add_systems(Update, dequeue::<Behavior>);
		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Marker<Idle>>());
	}
}
