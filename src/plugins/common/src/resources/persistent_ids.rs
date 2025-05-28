use crate::{components::persistent_id::PersistentId, traits::accessors::get::Get};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Debug, PartialEq)]
pub struct PersistentIds(HashMap<PersistentId, Entity>);

impl Get<PersistentId, Entity> for PersistentIds {
	fn get(&self, id: &PersistentId) -> Option<Entity> {
		self.0.get(id).copied()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_entity() {
		let id = PersistentId::default();
		let ids = PersistentIds(HashMap::from([(id, Entity::from_raw(42))]));

		let entity = ids.get(&id);

		assert_eq!(Some(Entity::from_raw(42)), entity);
	}
}
