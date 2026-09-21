use bevy::{ecs::entity::EntityHashSet, prelude::*};

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = AnchoredColliderTo, linked_spawn)]
pub(crate) struct AnchoredColliders(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = AnchoredColliders)]
#[require(Transform)]
pub(crate) struct AnchoredColliderTo(pub(crate) Entity);
