use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component, PartialEq, Eq, Hash, Debug, Clone, Copy, PartialOrd, Ord)]
#[require(Collider, Transform, ActiveEvents, ActiveCollisionTypes)]
pub struct ColliderRoot(pub Entity);
