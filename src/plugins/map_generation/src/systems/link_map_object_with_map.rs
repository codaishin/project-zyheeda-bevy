use crate::components::map::{
	Map,
	objects::{MapObject, MapObjectOf},
};
use bevy::prelude::*;
use common::{
	errors::{ErrorData, Level},
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::ZyheedaCommands,
};
use std::fmt::Display;

impl MapObject {
	pub(crate) fn link_with_map(
		mut commands: ZyheedaCommands,
		parents: Query<&ChildOf>,
		maps: Query<(Entity, &Map)>,
		spawners: Query<Entity, (With<Self>, Without<MapObjectOf>)>,
	) -> Result<(), Vec<MapError>> {
		let errors = spawners
			.iter()
			.filter_map(|entity| {
				let Some(map) = parents.iter_ancestors(entity).last() else {
					return Some(MapError::NoParentOf(entity));
				};
				let Ok((map, Map { .. })) = maps.get(map) else {
					return Some(MapError::NoMapOn(map));
				};

				commands.try_apply_on(&entity, |mut e| {
					e.try_insert(MapObjectOf(map));
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
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn link_with_map() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map = app.world_mut().spawn(Map::default()).id();
		let in_between = app.world_mut().spawn(ChildOf(map)).id();
		let spawner = app.world_mut().spawn((ChildOf(in_between), MapObject)).id();

		_ = app.world_mut().run_system_once(MapObject::link_with_map)?;

		assert_eq!(
			Some(&MapObjectOf(map)),
			app.world().entity(spawner).get::<MapObjectOf>(),
		);
		Ok(())
	}

	#[test]
	fn act_only_once() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map = app.world_mut().spawn(Map::default()).id();
		let spawner = app.world_mut().spawn((ChildOf(map), MapObject)).id();

		app.add_systems(
			Update,
			(
				MapObject::link_with_map.pipe(|In(_)| {}),
				IsChanged::<MapObjectOf>::detect,
			)
				.chain(),
		);
		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world()
				.entity(spawner)
				.get::<IsChanged::<MapObjectOf>>(),
		);
		Ok(())
	}

	#[test]
	fn return_no_parent() -> Result<(), RunSystemError> {
		let mut app = setup();
		let spawner = app.world_mut().spawn(MapObject).id();

		let result = app.world_mut().run_system_once(MapObject::link_with_map)?;

		assert_eq!(Err(vec![MapError::NoParentOf(spawner)]), result);
		Ok(())
	}

	#[test]
	fn return_map_missing_error() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((ChildOf(map), MapObject));

		let result = app.world_mut().run_system_once(MapObject::link_with_map)?;

		assert_eq!(Err(vec![MapError::NoMapOn(map)]), result);
		Ok(())
	}

	#[test]
	fn return_ok() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map = app.world_mut().spawn(Map::default()).id();
		app.world_mut().spawn((ChildOf(map), MapObject));

		let result = app.world_mut().run_system_once(MapObject::link_with_map)?;

		assert_eq!(Ok(()), result);
		Ok(())
	}
}
