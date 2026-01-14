use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::tools::Units;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[require(Transform, ActiveHooks = ActiveHooks::FILTER_INTERSECTION_PAIR)]
pub(crate) struct Hollow {
	pub(crate) radius: Units,
}
