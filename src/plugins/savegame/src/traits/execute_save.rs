use crate::errors::EntitySerializationErrors;
use bevy::ecs::world::EntityRef;

pub trait BufferComponents {
	fn buffer_components(&mut self, entity: EntityRef) -> Result<(), EntitySerializationErrors>;
}
