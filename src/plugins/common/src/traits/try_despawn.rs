pub mod commands;

use crate::components::persistent_entity::PersistentEntity;
use bevy::prelude::*;

pub trait TryDespawn {
	fn try_despawn(&mut self, entity: Entity);
}

pub trait TryDespawnPersistent {
	/// Use to despawn persistent entities. Does not panic when entity cannot be found.
	///
	/// Requires [`CommonPlugin`](crate::CommonPlugin) or `app.register_persistent_entities()`
	fn try_despawn_persistent(&mut self, entity: PersistentEntity);
}
