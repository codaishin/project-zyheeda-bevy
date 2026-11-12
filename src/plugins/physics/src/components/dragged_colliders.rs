use bevy::{ecs::entity::EntityHashSet, prelude::*};

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = DragColliderWith, linked_spawn)]
pub(crate) struct DraggedColliders(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = DraggedColliders)]
#[require(Transform)]
pub(crate) struct DragColliderWith(pub(crate) Entity);
