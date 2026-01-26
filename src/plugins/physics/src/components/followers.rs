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

impl Follow {
	pub(crate) fn with_offset(self, offset: Vec3) -> (Self, FollowWithOffset) {
		(self, FollowWithOffset(offset))
	}
}

#[derive(Component, Debug, PartialEq)]
pub(crate) struct FollowWithOffset(pub(crate) Vec3);
