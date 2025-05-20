pub mod commands;

use bevy::prelude::*;

pub trait TryDespawn {
	fn try_despawn(&mut self, entity: Entity);
}
