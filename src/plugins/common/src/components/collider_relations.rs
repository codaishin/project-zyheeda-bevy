use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component, PartialEq, Eq, Hash, Debug, Clone, Copy, PartialOrd, Ord)]
#[relationship(relationship_target = ChildColliders)]
#[require(Collider, Transform, ActiveEvents, ActiveCollisionTypes)]
pub struct ChildColliderOf(pub Entity);

#[derive(Component, PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord)]
#[relationship_target(relationship = ChildColliderOf)]
pub struct ChildColliders(Vec<Entity>);
