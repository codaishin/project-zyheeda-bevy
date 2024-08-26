use bevy::prelude::{default, Component, Entity};
use std::{collections::HashSet, fmt::Debug, marker::PhantomData};

#[derive(Component)]
pub(crate) struct ActedOnTargets<TActor: Component> {
	phantom_data: PhantomData<TActor>,
	pub(crate) entities: HashSet<Entity>,
}

impl<TActor: Component> Default for ActedOnTargets<TActor> {
	fn default() -> Self {
		Self {
			phantom_data: PhantomData,
			entities: default(),
		}
	}
}

impl<TActor: Component> Debug for ActedOnTargets<TActor> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ActedOnEntities")
			.field("phantom_data", &self.phantom_data)
			.field("entities", &self.entities)
			.finish()
	}
}

impl<TActor: Component> PartialEq for ActedOnTargets<TActor> {
	fn eq(&self, other: &Self) -> bool {
		self.phantom_data == other.phantom_data && self.entities == other.entities
	}
}

impl<TActor: Component> ActedOnTargets<TActor> {
	#[cfg(test)]
	pub(crate) fn new<const N: usize>(entities: [Entity; N]) -> Self {
		Self {
			entities: HashSet::from(entities),
			phantom_data: PhantomData,
		}
	}
}
