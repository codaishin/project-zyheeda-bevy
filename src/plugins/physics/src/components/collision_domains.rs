use bevy::prelude::*;
use common::components::persistent_entity::PersistentEntity;

#[derive(Component, PartialEq, Debug, Clone, Copy)]
#[component(immutable)]
#[require(PersistentEntity)]
pub(crate) enum Physical {
	Contact,
	Projection,
}

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct Interactive;
