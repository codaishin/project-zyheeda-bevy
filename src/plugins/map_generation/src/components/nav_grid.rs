use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub(crate) struct NavGridEntity {
	pub(crate) entity: Entity,
}

impl From<Entity> for NavGridEntity {
	fn from(entity: Entity) -> Self {
		Self { entity }
	}
}
