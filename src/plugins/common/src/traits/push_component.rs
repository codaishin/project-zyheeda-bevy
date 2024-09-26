use bevy::prelude::Entity;

pub trait PushComponent<TComponent> {
	fn push_component(&mut self, entity: Entity, component: TComponent);
}
