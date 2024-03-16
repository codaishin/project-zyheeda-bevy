pub mod commands;

use bevy::ecs::{bundle::Bundle, entity::Entity};

pub trait TryInsertOn {
	fn try_insert_on<TBundle: Bundle>(&mut self, entity: Entity, bundle: TBundle);
}
