use bevy::ecs::world::EntityRef;

pub trait BufferComponents {
	fn buffer_components(&mut self, entity: EntityRef);
}
