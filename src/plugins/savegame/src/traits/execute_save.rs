use bevy::ecs::{component::Component, world::EntityRef};
use serde::Serialize;

pub trait ExecuteSave {
	fn buffer<T>(&mut self, entity: EntityRef)
	where
		T: 'static + Component + Serialize;
}
