pub mod behavior;

use bevy::ecs::system::EntityCommands;

pub trait RemoveFromEntity {
	fn remove_from_entity(&self, entity: &mut EntityCommands);
}
