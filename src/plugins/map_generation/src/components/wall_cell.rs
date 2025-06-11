use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	blocker::{Blocker, Blockers},
	components::NoTarget,
};

#[derive(Component, Debug, PartialEq)]
#[require(
	Transform,
	Blockers = [Blocker::Physical],
	Collider = Collider::cuboid(0.5, 0.5, 0.5),
	NoTarget
)]
pub(crate) struct WallCell;
