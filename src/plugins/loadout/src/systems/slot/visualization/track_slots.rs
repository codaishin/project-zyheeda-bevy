use crate::components::slots::visualization::SlotVisualization;
use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;
use std::hash::Hash;

impl<TSlot> SlotVisualization<TSlot>
where
	TSlot: Eq + Hash + ThreadSafe,
{
	pub(crate) fn track_slots_for<TAgent>(
		mut agents: Query<(&TAgent, &mut Self)>,
		names: Query<(Entity, &Name), Added<Name>>,
		parents: Query<&ChildOf>,
	) where
		TAgent: Component + GetSlotDefinition<TSlot>,
	{
		for (entity, name) in names {
			let Some(parent) = parents.iter_ancestors(entity).find(|p| agents.contains(*p)) else {
				continue;
			};
			let Ok((agent, mut slots)) = agents.get_mut(parent) else {
				continue;
			};
			let Some(slot) = agent.get_slot_definition(name) else {
				continue;
			};
			slots.slots.insert(slot, entity);
		}
	}
}

pub(crate) trait GetSlotDefinition<T> {
	fn get_slot_definition(&self, name: &str) -> Option<T>;
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq, Eq, Hash)]
	struct _Key;

	#[derive(Component)]
	#[require(SlotVisualization<_Key>)]
	struct _Agent;

	impl GetSlotDefinition<_Key> for _Agent {
		fn get_slot_definition(&self, key: &str) -> Option<_Key> {
			match key {
				"key" => Some(_Key),
				_ => None,
			}
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, SlotVisualization::<_Key>::track_slots_for::<_Agent>);

		app
	}

	#[test]
	fn insert_child_entity() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Agent).id();
		let child = app
			.world_mut()
			.spawn((ChildOf(agent), Name::from("key")))
			.id();

		app.update();

		assert_eq!(
			Some(&SlotVisualization::from([(_Key, child)])),
			app.world().entity(agent).get::<SlotVisualization<_Key>>()
		);
	}

	#[test]
	fn insert_nested_child_entity() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Agent).id();
		let child = app.world_mut().spawn(ChildOf(agent)).id();
		let child_child = app
			.world_mut()
			.spawn((ChildOf(child), Name::from("key")))
			.id();

		app.update();

		assert_eq!(
			Some(&SlotVisualization::from([(_Key, child_child)])),
			app.world().entity(agent).get::<SlotVisualization<_Key>>()
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let agent = app.world_mut().spawn(_Agent).id();
		app.world_mut().spawn((ChildOf(agent), Name::from("key")));

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.insert(SlotVisualization::<_Key>::default());
		app.update();

		assert_eq!(
			Some(&SlotVisualization::default()),
			app.world().entity(agent).get::<SlotVisualization<_Key>>()
		);
	}
}
