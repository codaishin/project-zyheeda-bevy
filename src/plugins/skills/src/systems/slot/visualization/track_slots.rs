use crate::components::slots::visualization::SlotVisualization;
use bevy::prelude::*;
use common::{
	tools::bone::Bone,
	traits::{
		accessors::get::{AssociatedSystemParam, GetFromSystemParam},
		mapper::Mapper,
		thread_safe::ThreadSafe,
	},
};
use std::hash::Hash;

impl<TKey> SlotVisualization<TKey>
where
	TKey: Eq + Hash + ThreadSafe,
{
	pub(crate) fn track_slots_for<TAgent>(
		mut agents: Query<(&TAgent, &mut Self)>,
		names: Query<(Entity, &Name), Added<Name>>,
		parents: Query<&ChildOf>,
		param: AssociatedSystemParam<TAgent, ()>,
	) where
		TAgent: Component + GetFromSystemParam<()>,
		for<'i> TAgent::TItem<'i>: Mapper<Bone<'i>, Option<TKey>>,
	{
		for (entity, name) in names {
			let Some(parent) = parents.iter_ancestors(entity).find(|p| agents.contains(*p)) else {
				continue;
			};
			let Ok((agent, mut slots)) = agents.get_mut(parent) else {
				continue;
			};
			let Some(mapper) = agent.get_from_param(&(), &param) else {
				continue;
			};
			let Some(key) = mapper.map(Bone(name.as_str())) else {
				continue;
			};
			slots.slots.insert(key, entity);
		}
	}
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

	impl GetFromSystemParam<()> for _Agent {
		type TParam<'w, 's> = ();
		type TItem<'i> = _Config;

		fn get_from_param(&self, _: &(), _: &()) -> Option<_Config> {
			Some(_Config)
		}
	}

	struct _Config;

	impl Mapper<Bone<'_>, Option<_Key>> for _Config {
		fn map(&self, Bone(bone): Bone<'_>) -> Option<_Key> {
			match bone {
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
