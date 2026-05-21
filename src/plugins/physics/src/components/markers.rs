use bevy::prelude::*;
use common::components::persistent_entity::PersistentEntity;

#[derive(Component, PartialEq, Debug, Default, Clone, Copy)]
#[component(immutable)]
#[require(PersistentEntity)]
pub(crate) struct Physical;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct Interactive;
