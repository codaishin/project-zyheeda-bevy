use std::fmt::Debug;

use crate::components::{Idle, Queue};
use bevy::prelude::{Commands, Component, Entity, Query, With};

fn match_first<TBehavior: Copy, TComponent: TryFrom<TBehavior>>(
	queue: &Queue<TBehavior>,
) -> Option<TComponent> {
	queue.0.get(0).and_then(|b| TComponent::try_from(*b).ok())
}

#[allow(clippy::type_complexity)]
pub fn dequeue<
	TAgent: Component,
	TBehavior: Copy + Send + Sync + 'static,
	TComponent: Component + TryFrom<TBehavior> + Debug,
>(
	mut commands: Commands,
	mut agents: Query<(Entity, &mut Queue<TBehavior>), (With<TAgent>, With<Idle<TBehavior>>)>,
) {
	for (agent, mut queue) in agents.iter_mut() {
		let mut agent = commands.entity(agent);

		agent.remove::<TComponent>();
		if let Some(component) = match_first::<TBehavior, TComponent>(&queue) {
			queue.0.pop_front();
			agent.insert(component);
			agent.remove::<Idle<TBehavior>>();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Idle;
	use bevy::prelude::{App, Update};
	use std::collections::VecDeque;

	#[derive(Clone, Copy)]
	enum Behavior {
		Sing,
		Dance,
	}

	#[derive(Component, Debug)]
	struct Sing;

	impl TryFrom<Behavior> for Sing {
		type Error = ();

		fn try_from(value: Behavior) -> Result<Self, Self::Error> {
			match value {
				Behavior::Sing => Ok(Sing),
				_ => Err(()),
			}
		}
	}

	#[derive(Component)]
	struct Agent;

	#[test]
	fn pop_first_behavior_to_agent() {
		let mut app = App::new();
		let queue = Queue(VecDeque::from([Behavior::Sing]));
		let agent = Agent;
		let idle = Idle::<Behavior>::new();

		let agent = app.world.spawn((agent, queue, idle)).id();
		app.add_systems(Update, dequeue::<Agent, Behavior, Sing>);
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
		let queue = Queue(VecDeque::from([Behavior::Sing]));
		let agent = Agent;

		let agent = app.world.spawn((agent, queue)).id();
		app.add_systems(Update, dequeue::<Agent, Behavior, Sing>);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<Behavior>>().unwrap();

		assert_eq!((false, 1), (agent.contains::<Sing>(), queue.0.len()));
	}

	#[test]
	fn remove_component_when_idling() {
		let mut app = App::new();
		let queue = Queue(VecDeque::<Behavior>::from([]));
		let agent = Agent;
		let idle = Idle::<Behavior>::new();
		let sing = Sing;

		let agent = app.world.spawn((agent, queue, sing, idle)).id();
		app.add_systems(Update, dequeue::<Agent, Behavior, Sing>);
		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Sing>());
	}

	#[test]
	fn do_not_pop_when_bundle_cannot_be_created_from_behavior() {
		let mut app = App::new();
		let queue = Queue(VecDeque::from([Behavior::Dance]));
		let agent = Agent;
		let idle = Idle::<Behavior>::new();

		let agent = app.world.spawn((agent, queue, idle)).id();
		app.add_systems(Update, dequeue::<Agent, Behavior, Sing>);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<Behavior>>().unwrap();

		assert_eq!((false, 1), (agent.contains::<Sing>(), queue.0.len()));
	}
}
