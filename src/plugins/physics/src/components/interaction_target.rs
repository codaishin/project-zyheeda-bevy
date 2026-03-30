use bevy::prelude::*;
use common::components::persistent_entity::PersistentEntity;

/// Marks an entity as the target for interactions like damaging effects, healing, etc.
#[derive(Component, PartialEq, Debug, Default, Clone, Copy)]
#[component(immutable)]
#[require(PersistentEntity)]
pub(crate) struct InteractionTarget;
