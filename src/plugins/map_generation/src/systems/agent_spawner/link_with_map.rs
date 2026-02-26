use crate::components::{
	agent_spawner::{AgentSpawner, SpawnerActive},
	map::{Map, objects::MapObjectOfPersistent},
};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	errors::{ErrorData, Level},
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::ZyheedaCommands,
};
use std::fmt::Display;

impl AgentSpawner {
	pub(crate) fn link_with_map(
		mut commands: ZyheedaCommands,
		parents: Query<&ChildOf>,
		maps: Query<(&PersistentEntity, &Map)>,
		spawners: Query<Entity, (With<Self>, Without<MapObjectOfPersistent>)>,
	) -> Result<(), Vec<MapError>> {
		let errors = spawners
			.iter()
			.filter_map(|entity| {
				let Some(map) = parents.iter_ancestors(entity).last() else {
					return Some(MapError::NoParentOf(entity));
				};
				let Ok((persistent_map, Map { created_from_save })) = maps.get(map) else {
					return Some(MapError::NoMapOn(map));
				};

				commands.try_apply_on(&entity, |mut e| {
					e.try_insert(MapObjectOfPersistent(*persistent_map));
					if *created_from_save {
						e.try_remove::<SpawnerActive>();
					}
				});

				None
			})
			.collect::<Vec<_>>();

		if !errors.is_empty() {
			return Err(errors);
		}

		Ok(())
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum MapError {
	NoParentOf(Entity),
	NoMapOn(Entity),
}

impl Display for MapError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MapError::NoParentOf(entity) => write!(f, "{entity:?}: has no parent"),
			MapError::NoMapOn(entity) => write!(f, "{entity:?}: `Map` missing"),
		}
	}
}

impl ErrorData for MapError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Map Error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{
		agent_spawner::SpawnerActive,
		map::{Map, objects::MapObjectOfPersistent},
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		components::persistent_entity::PersistentEntity,
		traits::handles_map_generation::AgentType,
	};
	use test_case::test_case;
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn link_with_persistent_map() -> Result<(), RunSystemError> {
		let mut app = setup();
		let persistent = PersistentEntity::default();
		let map = app.world_mut().spawn((Map::default(), persistent)).id();
		let in_between = app.world_mut().spawn(ChildOf(map)).id();
		let spawner = app
			.world_mut()
			.spawn((ChildOf(in_between), AgentSpawner(AgentType::Player)))
			.id();

		_ = app
			.world_mut()
			.run_system_once(AgentSpawner::link_with_map)?;

		assert_eq!(
			Some(&MapObjectOfPersistent(persistent)),
			app.world().entity(spawner).get::<MapObjectOfPersistent>(),
		);
		Ok(())
	}

	#[test_case(Map {created_from_save: true}, false; "map created from save")]
	#[test_case(Map {created_from_save: false}, true; "map not created from save")]
	fn control_active_marker(map: Map, is_active: bool) -> Result<(), RunSystemError> {
		let mut app = setup();
		let map = app.world_mut().spawn(map).id();
		let spawner = app
			.world_mut()
			.spawn((ChildOf(map), AgentSpawner(AgentType::Player)))
			.id();

		_ = app
			.world_mut()
			.run_system_once(AgentSpawner::link_with_map)?;

		assert_eq!(
			is_active,
			app.world().entity(spawner).contains::<SpawnerActive>(),
		);
		Ok(())
	}

	#[test]
	fn act_only_once() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map = app.world_mut().spawn(Map::default()).id();
		let spawner = app
			.world_mut()
			.spawn((ChildOf(map), AgentSpawner(AgentType::Player)))
			.id();

		app.add_systems(
			Update,
			(
				AgentSpawner::link_with_map.pipe(|In(_)| {}),
				IsChanged::<MapObjectOfPersistent>::detect,
			)
				.chain(),
		);
		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world()
				.entity(spawner)
				.get::<IsChanged::<MapObjectOfPersistent>>(),
		);
		Ok(())
	}

	#[test]
	fn return_no_parent() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner = app.world_mut().spawn(AgentSpawner(AgentType::Player)).id();

		let result = app
			.world_mut()
			.run_system_once(AgentSpawner::link_with_map)?;

		assert_eq!(Err(vec![MapError::NoParentOf(spawner)]), result);
		Ok(())
	}

	#[test]
	fn return_map_missing_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map = app.world_mut().spawn_empty().id();
		app.world_mut()
			.spawn((ChildOf(map), AgentSpawner(AgentType::Player)));

		let result = app
			.world_mut()
			.run_system_once(AgentSpawner::link_with_map)?;

		assert_eq!(Err(vec![MapError::NoMapOn(map)]), result);
		Ok(())
	}

	#[test]
	fn return_ok() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map = app.world_mut().spawn(Map::default()).id();
		app.world_mut()
			.spawn((ChildOf(map), AgentSpawner(AgentType::Player)));

		let result = app
			.world_mut()
			.run_system_once(AgentSpawner::link_with_map)?;

		assert_eq!(Ok(()), result);
		Ok(())
	}
}
