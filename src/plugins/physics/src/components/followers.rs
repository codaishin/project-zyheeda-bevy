use bevy::{ecs::entity::EntityHashSet, prelude::*};

/// Used for early selective transform propagation
///
/// Not suited for nested or [`ChildOf`] relationships
#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = Follow)]
pub(crate) struct Followers(EntityHashSet);

/// Used for early selective transform propagation
///
/// Not suited for nested or [`ChildOf`] relationships
#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = Followers)]
#[require(GlobalTransform)]
pub(crate) struct Follow(pub(crate) Entity);
