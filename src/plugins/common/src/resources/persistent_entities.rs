use crate::{
	components::persistent_entity::PersistentEntity,
	errors::{Error, Level},
};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Debug, PartialEq, Default)]
pub struct PersistentEntities {
	pub(crate) entities: HashMap<PersistentEntity, Entity>,
	pub(crate) errors: Vec<LookupError>,
}

impl PersistentEntities {
	/// Attempt to retrieve an [`Entity`] for the given [`PersistentEntity`].
	///
	/// Failures are logged by the [`crate::CommonPlugin`]. `self` is mutable to allow collection of
	/// lookup errors, which are used for logging via a dedicated system.
	pub fn get_entity(&mut self, persistent_entity: &PersistentEntity) -> Option<Entity> {
		let Some(entity) = self.entities.get(persistent_entity) else {
			self.errors.push(LookupError(*persistent_entity));
			return None;
		};

		Some(*entity)
	}
}

#[cfg(test)]
impl<const N: usize> From<[(PersistentEntity, Entity); N]> for PersistentEntities {
	fn from(entities: [(PersistentEntity, Entity); N]) -> Self {
		Self {
			entities: HashMap::from(entities),
			errors: vec![],
		}
	}
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub(crate) struct LookupError(pub(crate) PersistentEntity);

impl From<LookupError> for Error {
	fn from(LookupError(persistent_entity): LookupError) -> Self {
		Self::Single {
			msg: format!("{persistent_entity:?}: no matching entity found"),
			lvl: Level::Warning,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_entity() {
		let target = Entity::from_raw(42);
		let persistent_entity = PersistentEntity::default();
		let mut persistent_entities = PersistentEntities::from([(persistent_entity, target)]);

		let entity = persistent_entities.get_entity(&persistent_entity);

		assert_eq!(Some(target), entity);
	}

	#[test]
	fn collect_lookup_miss() {
		let persistent_entity = PersistentEntity::default();
		let mut persistent_entities = PersistentEntities::from([]);

		persistent_entities.get_entity(&persistent_entity);

		assert_eq!(
			vec![LookupError(persistent_entity)],
			persistent_entities.errors
		);
	}
}
