use bevy::ecs::world::EntityRef;

pub trait ExecuteSave {
	fn execute_save<T>(&mut self, entity: EntityRef)
	where
		T: 'static;
}
