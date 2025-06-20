use crate::errors::EntitySerializationErrors;
use bevy::ecs::world::EntityRef;

pub trait WriteBuffer {
	fn write_buffer(&mut self, entity: EntityRef) -> Result<(), EntitySerializationErrors>;
}
