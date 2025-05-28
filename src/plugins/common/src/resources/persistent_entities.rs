use crate::{components::persistent_entity::PersistentEntity, traits::accessors::get::Get};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Debug, PartialEq, Default)]
pub struct PersistentEntities(pub(crate) HashMap<PersistentEntity, Entity>);

impl Get<PersistentEntity, Entity> for PersistentEntities {
	fn get(&self, id: &PersistentEntity) -> Option<Entity> {
		self.0.get(id).copied()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_entity() {
		let target = Entity::from_raw(42);
		let persistent_entity = PersistentEntity::default();
		let persistent_entities = PersistentEntities(HashMap::from([(persistent_entity, target)]));

		let entity = persistent_entities.get(&persistent_entity);

		assert_eq!(Some(target), entity);
	}
}
