use bevy::prelude::Entity;

pub trait Track<TComponent> {
	fn track(&mut self, entity: Entity);
}

pub trait IsTracking<TComponent> {
	#[must_use]
	fn is_tracking(&self, entity: Entity) -> bool;
}

pub trait Untrack<TComponent> {
	fn untrack(&mut self, entity: Entity);
}
