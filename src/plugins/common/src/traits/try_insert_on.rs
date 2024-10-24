pub mod commands;

use bevy::prelude::*;

pub trait TryInsertOn {
	fn try_insert_on<TBundle: Bundle>(&mut self, entity: Entity, bundle: TBundle);
}

impl<T> TryInsertOn for In<T>
where
	T: TryInsertOn,
{
	fn try_insert_on<TBundle: Bundle>(&mut self, entity: Entity, bundle: TBundle) {
		self.0.try_insert_on(entity, bundle);
	}
}
