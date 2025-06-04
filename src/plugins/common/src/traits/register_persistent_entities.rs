use crate::{resources::persistent_entities::PersistentEntities, systems::log::log_many};
use bevy::prelude::*;

pub trait RegisterPersistentEntities {
	fn register_persistent_entities(&mut self) -> &mut Self;
}

impl RegisterPersistentEntities for App {
	fn register_persistent_entities(&mut self) -> &mut Self {
		self.init_resource::<PersistentEntities>()
			.add_observer(PersistentEntities::update)
			.add_systems(
				Update,
				PersistentEntities::drain_lookup_errors.pipe(log_many),
			)
	}
}
