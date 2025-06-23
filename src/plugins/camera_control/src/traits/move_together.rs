use bevy::prelude::*;
use common::components::persistent_entity::PersistentEntity;

pub(crate) trait MoveTogether {
	fn entity(&self) -> Option<PersistentEntity>;
	fn move_together_with(&mut self, transform: &mut Transform, new_position: Vec3);
}
