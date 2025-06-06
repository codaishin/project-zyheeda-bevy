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
	Collider = Self::collider(),
	NoTarget
)]
pub(crate) struct WallCell;

impl WallCell {
	fn collider() -> Collider {
		Collider::cuboid(0.5, 0.5, 0.5)
	}
}
