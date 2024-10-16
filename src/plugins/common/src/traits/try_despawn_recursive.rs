pub mod commands;

use bevy::prelude::*;

pub trait TryDespawnRecursive {
	fn try_despawn_recursive(&mut self, entity: Entity);
}
