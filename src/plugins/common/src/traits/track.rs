use bevy::prelude::Entity;

pub trait Track<TComponent> {
	fn track(&mut self, entity: Entity);
}
