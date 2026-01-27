use crate::observers::lifetime::insert_on::LifetimeRoot;
use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::traits::accessors::get::GetProperty;

/// Used for early selective transform propagation
///
/// Must not be used nested or in [`ChildOf`] relationships
#[derive(Component, Debug, PartialEq, Default)]
#[relationship_target(relationship = Follow)]
#[require(Transform, FollowStateDirty)]
pub(crate) struct Followers(EntityHashSet);

/// Used for early selective transform propagation
///
/// Must not be used nested or in [`ChildOf`] relationships
#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = Followers)]
#[require(Transform, FollowTransform)]
pub(crate) struct Follow(pub(crate) Entity);

impl GetProperty<LifetimeRoot> for Follow {
	fn get_property(&self) -> LifetimeRoot {
		LifetimeRoot::Transient(self.0)
	}
}

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct FollowTransform {
	pub(crate) translation: Vec3,
	pub(crate) rotation: Quat,
}

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct FollowStateDirty;
