mod commands;

use bevy::prelude::Entity;

pub trait TryComplexInsert<T> {
	fn try_complex_insert(&mut self, entity: Entity, value: T);
}
