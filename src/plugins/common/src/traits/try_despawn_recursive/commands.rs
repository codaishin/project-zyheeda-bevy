use super::TryDespawnRecursive;
use bevy::prelude::*;

impl<'w, 's> TryDespawnRecursive for Commands<'w, 's> {
	fn try_despawn_recursive(&mut self, entity: Entity) {
		let Some(entity) = self.get_entity(entity) else {
			return;
		};
		entity.despawn_recursive();
	}
}
