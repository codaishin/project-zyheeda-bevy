use crate::{resources::persistent_entities::PersistentEntities, systems::log::OnError};
use bevy::prelude::*;

pub trait RegisterPersistentEntities {
	fn register_persistent_entities(&mut self) -> &mut Self;
}

impl RegisterPersistentEntities for App {
	fn register_persistent_entities(&mut self) -> &mut Self {
		self.init_resource::<PersistentEntities>()
			.add_observer(PersistentEntities::insert_entity)
			.add_observer(PersistentEntities::remove_entity)
			.add_observer(PersistentEntities::despawn_entity)
			.add_systems(
				Update,
				PersistentEntities::drain_lookup_errors.pipe(OnError::log_many),
			)
	}
}
