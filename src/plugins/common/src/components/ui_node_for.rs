use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq)]
pub struct UiNodeFor<T> {
	pub owner: Entity,
	owner_type: PhantomData<T>,
}

impl<T> UiNodeFor<T> {
	pub fn with(owner: Entity) -> Self {
		Self {
			owner,
			owner_type: PhantomData,
		}
	}
}
