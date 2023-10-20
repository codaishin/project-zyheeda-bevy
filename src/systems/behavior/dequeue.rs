use crate::components::{Group, Queue};
use bevy::prelude::{Bundle, Commands, Component, Entity, Query, With, Without};

fn match_first<TBehavior: Copy, TBundle: TryFrom<TBehavior>>(
	queue: &Queue<TBehavior>,
) -> Option<TBundle> {
	queue.0.get(0).and_then(|b| TBundle::try_from(*b).ok())
}

#[allow(clippy::type_complexity)]
pub fn dequeue<
	TAgent: Component,
	TBehavior: Copy + Send + Sync + 'static,
	TBundle: Bundle + TryFrom<TBehavior>,
>(
	mut commands: Commands,
	mut agents: Query<(Entity, &mut Queue<TBehavior>), (With<TAgent>, Without<Group<TBehavior>>)>,
) {
	for (agent, mut queue) in agents.iter_mut() {
		let mut agent = commands.entity(agent);

		agent.remove::<TBundle>();
		if let Some(bundle) = match_first::<TBehavior, TBundle>(&queue) {
			let group = Group::<TBehavior>::new();
			queue.0.pop_front();
			agent.insert((bundle, group));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Group;
	use bevy::{
		prelude::{App, Update},
		utils::default,
	};
	use std::collections::VecDeque;

	#[derive(Clone, Copy)]
	enum Behavior {
		Sing,
		Dance,
	}

	#[derive(Component)]
	struct Sing;

	#[derive(Component)]
	struct Pop;

	impl TryFrom<Behavior> for (Sing, Pop) {
		type Error = ();

		fn try_from(value: Behavior) -> Result<Self, Self::Error> {
			match value {
				Behavior::Sing => Ok((Sing, Pop)),
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

		let agent = app.world.spawn((agent, queue)).id();
		app.add_systems(Update, dequeue::<Agent, Behavior, (Sing, Pop)>);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<Behavior>>().unwrap();

		assert_eq!(
			(true, true, true, 0),
			(
				agent.contains::<Sing>(),
				agent.contains::<Pop>(),
				agent.contains::<Group<Behavior>>(),
				queue.0.len()
			)
		);
	}

	#[test]
	fn do_not_pop_when_something_is_running() {
		let mut app = App::new();
		let queue = Queue(VecDeque::from([Behavior::Sing]));
		let agent = Agent;
		let running: Group<Behavior> = default();

		let agent = app.world.spawn((agent, queue, running)).id();
		app.add_systems(Update, dequeue::<Agent, Behavior, (Sing, Pop)>);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<Behavior>>().unwrap();

		assert_eq!(
			(false, false, true, 1),
			(
				agent.contains::<Sing>(),
				agent.contains::<Pop>(),
				agent.contains::<Group<Behavior>>(),
				queue.0.len()
			)
		);
	}

	#[test]
	fn remove_bundle_when_not_running() {
		let mut app = App::new();
		let queue = Queue(VecDeque::<Behavior>::from([]));
		let agent = Agent;
		let sing = Sing;

		let agent = app.world.spawn((agent, queue, sing)).id();
		app.add_systems(Update, dequeue::<Agent, Behavior, (Sing, Pop)>);
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false),
			(agent.contains::<Sing>(), agent.contains::<Pop>())
		);
	}

	#[test]
	fn do_not_pop_when_bundle_cannot_be_created_from_behavior() {
		let mut app = App::new();
		let queue = Queue(VecDeque::from([Behavior::Dance]));
		let agent = Agent;

		let agent = app.world.spawn((agent, queue)).id();
		app.add_systems(Update, dequeue::<Agent, Behavior, (Sing, Pop)>);
		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<Behavior>>().unwrap();

		assert_eq!(
			(false, false, 1),
			(
				agent.contains::<Sing>(),
				agent.contains::<Pop>(),
				queue.0.len()
			)
		);
	}
}
