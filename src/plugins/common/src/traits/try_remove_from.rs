pub mod commands;

use bevy::ecs::{bundle::Bundle, entity::Entity};

pub trait TryRemoveFrom {
	fn try_remove_from<TBundle: Bundle>(&mut self, entity: Entity);
}
