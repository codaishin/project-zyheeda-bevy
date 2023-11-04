pub mod behavior;

use bevy::ecs::system::EntityCommands;

pub trait InsertIntoEntity {
	fn insert_into_entity(self, entity: &mut EntityCommands);
}
