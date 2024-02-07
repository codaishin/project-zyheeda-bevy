use crate::traits::prefab::SpawnPrefab;
use bevy::ecs::{
	component::Component,
	entity::Entity,
	query::Added,
	system::{Commands, Query, Res, Resource},
};

pub fn instantiate<TAgent: Component, TPrefab: Resource + SpawnPrefab<TAgent>>(
	mut commands: Commands,
	prefab: Res<TPrefab>,
	agents: Query<Entity, Added<TAgent>>,
) {
	for agent in &agents {
		let agent = &mut commands.entity(agent);
		prefab.spawn_prefab(agent);
	}
}

#[cfg(test)]
mod tests {
	use bevy::{
		app::{App, Update},
		ecs::system::EntityCommands,
		hierarchy::{BuildChildren, Parent},
	};

	use super::*;

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, Debug, PartialEq)]
	struct _SpawnedFromPrefab;

	#[derive(Resource)]
	struct _Prefab;

	impl SpawnPrefab<_Agent> for _Prefab {
		fn spawn_prefab(&self, parent: &mut EntityCommands) {
			parent.with_children(|parent| {
				parent.spawn(_SpawnedFromPrefab);
			});
		}
	}

	fn setup() -> App {
		let mut app = App::new();
		app.insert_resource(_Prefab);
		app.add_systems(Update, instantiate::<_Agent, _Prefab>);

		app
	}

	#[test]
	fn spawn_prefab() {
		let mut app = setup();

		let agent = app.world.spawn(_Agent).id();

		app.update();

		let children = app
			.world
			.iter_entities()
			.filter_map(|entity| Some((entity, entity.get::<Parent>()?)))
			.filter(|(_, parent)| parent.get() == agent)
			.filter_map(|(entity, _)| entity.get::<_SpawnedFromPrefab>())
			.collect::<Vec<_>>();

		assert_eq!(vec![&_SpawnedFromPrefab], children);
	}

	#[test]
	fn spawn_prefab_only_when_agent_present() {
		let mut app = setup();

		let agent = app.world.spawn_empty().id();

		app.update();

		let children = app
			.world
			.iter_entities()
			.filter_map(|entity| Some((entity, entity.get::<Parent>()?)))
			.filter(|(_, parent)| parent.get() == agent)
			.filter_map(|(entity, _)| entity.get::<_SpawnedFromPrefab>())
			.collect::<Vec<_>>();

		assert!(children.is_empty());
	}

	#[test]
	fn spawn_prefab_only_once() {
		let mut app = setup();

		let agent = app.world.spawn(_Agent).id();

		app.update();
		app.update();

		let children = app
			.world
			.iter_entities()
			.filter_map(|entity| Some((entity, entity.get::<Parent>()?)))
			.filter(|(_, parent)| parent.get() == agent)
			.filter_map(|(entity, _)| entity.get::<_SpawnedFromPrefab>())
			.collect::<Vec<_>>();

		assert_eq!(vec![&_SpawnedFromPrefab], children);
	}
}
