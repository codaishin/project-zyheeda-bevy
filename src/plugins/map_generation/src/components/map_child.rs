use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub(crate) struct MapChild<T> {
	pub(crate) entity: Entity,
	_p: PhantomData<T>,
}

impl<T> From<Entity> for MapChild<T> {
	fn from(entity: Entity) -> Self {
		Self {
			entity,
			_p: PhantomData,
		}
	}
}
