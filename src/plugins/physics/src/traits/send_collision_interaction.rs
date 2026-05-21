use bevy::prelude::*;

pub(crate) trait PushInteractingColliders {
	fn push_interacting_colliders(&mut self, actor: Entity, target: Entity);
}
