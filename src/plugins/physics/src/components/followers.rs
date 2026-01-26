use bevy::{ecs::entity::EntityHashSet, prelude::*};

/// Used for early selective transform propagation
///
/// Must not be used nested or in [`ChildOf`] relationships
#[derive(Component, Debug, PartialEq, Default)]
#[relationship_target(relationship = Follow)]
pub(crate) struct Followers(EntityHashSet);

/// Used for early selective transform propagation
///
/// Must not be used nested or in [`ChildOf`] relationships
#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = Followers)]
#[require(GlobalTransform)]
pub(crate) struct Follow(pub(crate) Entity);

#[derive(Component, Debug, PartialEq)]
pub(crate) struct FollowWithOffset(pub(crate) Vec3);
