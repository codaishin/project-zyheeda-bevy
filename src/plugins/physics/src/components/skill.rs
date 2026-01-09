use bevy::prelude::*;
use common::components::persistent_entity::PersistentEntity;

#[derive(Component, Debug, PartialEq)]
#[require(PersistentEntity)]
pub(crate) struct Skill {
	pub(crate) contact: Entity,
	pub(crate) projection: Entity,
}
