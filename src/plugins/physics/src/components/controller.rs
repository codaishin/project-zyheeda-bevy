use bevy::{ecs::entity::EntityHashSet, prelude::*};

#[derive(Component, Debug, PartialEq, Default)]
#[relationship_target(relationship = ControllerOf, linked_spawn)]
pub(crate) struct Controllers(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = Controllers)]
pub(crate) struct ControllerOf(pub(crate) Entity);
