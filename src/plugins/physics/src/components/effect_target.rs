use bevy::prelude::*;
use common::components::persistent_entity::PersistentEntity;

/// Marks an entity as the target for effects like damage, healing, etc.
#[derive(Component, PartialEq, Debug, Default, Clone, Copy)]
#[component(immutable)]
#[require(PersistentEntity)]
pub(crate) struct EffectTarget;
