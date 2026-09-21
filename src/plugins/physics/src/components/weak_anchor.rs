use bevy::{ecs::entity::EntityHashSet, prelude::*};

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = WeakAnchoredTo, linked_spawn)]
pub(crate) struct WeakAnchors(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = WeakAnchors)]
#[require(Transform)]
pub(crate) struct WeakAnchoredTo(pub(crate) Entity);
