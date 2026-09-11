use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(Velocity, GlobalTransform)]
pub(crate) struct Projectile {
	pub(crate) leading_edge: Units,
}
