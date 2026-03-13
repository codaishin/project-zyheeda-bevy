use crate::components::map::{
	Map,
	objects::{MapObjectOf, PersistentMapObject},
};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	errors::{ErrorData, Level},
	traits::accessors::get::{Get, TryApplyOn},
	zyheeda_commands::ZyheedaCommands,
};
use std::fmt::Display;

impl PersistentMapObject {
	pub(crate) fn link_with_map(
		mut commands: ZyheedaCommands,
		map_objects: Query<(Entity, &Self), Without<MapObjectOf>>,
		maps: Query<(), With<Map>>,
	) -> Result<(), Vec<PersistentMapObjectError>> {
		let mut errors = vec![];

		for (entity, PersistentMapObject { map }) in map_objects {
			let Some(map) = commands.get(map) else {
				errors.push(PersistentMapObjectError::MissingMapEntity { map: *map });
				continue;
			};

			if !maps.contains(map) {
				errors.push(PersistentMapObjectError::MapEntityIsNotAMap { map });
				continue;
			}

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(MapObjectOf(map));
			});
		}

		if !errors.is_empty() {
			return Err(errors);
		}

		Ok(())
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum PersistentMapObjectError {
	MissingMapEntity { map: PersistentEntity },
	MapEntityIsNotAMap { map: Entity },
}

impl Display for PersistentMapObjectError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PersistentMapObjectError::MissingMapEntity { map } => {
				write!(f, "Map with entity ({map:?}) does not exist")
			}
			PersistentMapObjectError::MapEntityIsNotAMap { map } => {
				write!(f, "Entity ({map:?}) is not a map")
			}
		}
	}
}

impl ErrorData for PersistentMapObjectError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl std::fmt::Display {
		"Persistent Map Object Error"
	}

	fn into_details(self) -> impl std::fmt::Display {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::map::{Map, objects::MapObjectOf};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{CommonPlugin, components::persistent_entity::PersistentEntity};
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin);

		app
	}

	#[test]
	fn link_with_map() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map_persistent = PersistentEntity::default();
		let map = app.world_mut().spawn((Map::default(), map_persistent)).id();
		let entity = app
			.world_mut()
			.spawn(PersistentMapObject {
				map: map_persistent,
			})
			.id();

		let result = app
			.world_mut()
			.run_system_once(PersistentMapObject::link_with_map)?;

		assert_eq!(
			(Some(&MapObjectOf(map)), Ok(())),
			(app.world().entity(entity).get::<MapObjectOf>(), result),
		);
		Ok(())
	}

	#[test]
	fn error_when_map_entity_not_found() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map = PersistentEntity::default();
		let entity = app.world_mut().spawn(PersistentMapObject { map }).id();

		let result = app
			.world_mut()
			.run_system_once(PersistentMapObject::link_with_map)?;

		assert_eq!(
			(
				None,
				Err(vec![PersistentMapObjectError::MissingMapEntity { map }])
			),
			(app.world().entity(entity).get::<MapObjectOf>(), result),
		);
		Ok(())
	}

	#[test]
	fn error_when_map_entity_has_no_map() -> Result<(), RunSystemError> {
		let mut app = setup();
		let map_persistent = PersistentEntity::default();
		let map = app.world_mut().spawn(map_persistent).id();
		let entity = app
			.world_mut()
			.spawn(PersistentMapObject {
				map: map_persistent,
			})
			.id();

		let result = app
			.world_mut()
			.run_system_once(PersistentMapObject::link_with_map)?;

		assert_eq!(
			(
				None,
				Err(vec![PersistentMapObjectError::MapEntityIsNotAMap { map }])
			),
			(app.world().entity(entity).get::<MapObjectOf>(), result),
		);
		Ok(())
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let map_persistent = PersistentEntity::default();
		app.world_mut().spawn((Map::default(), map_persistent));
		let entity = app
			.world_mut()
			.spawn(PersistentMapObject {
				map: map_persistent,
			})
			.id();

		app.add_systems(
			Update,
			(
				PersistentMapObject::link_with_map.pipe(|In(_)| {}),
				IsChanged::<MapObjectOf>::detect,
			)
				.chain(),
		);
		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<MapObjectOf>>(),
		);
	}
}
