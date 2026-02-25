use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::tools::Units;

#[derive(Component, Debug, PartialEq)]
#[require(Velocity, GlobalTransform)]
pub(crate) struct PreventTunneling {
	pub(crate) leading_edge: Units,
}
