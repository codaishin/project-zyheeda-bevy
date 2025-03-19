use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	blocker::{Blocker, BlockerInsertCommand},
	components::NoTarget,
};

#[derive(Component, Debug, PartialEq)]
#[require(
	Transform,
	BlockerInsertCommand(Self::blocker),
	Collider(Self::collider),
	NoTarget
)]
pub(crate) struct WallCell;

impl WallCell {
	fn blocker() -> BlockerInsertCommand {
		Blocker::insert([Blocker::Physical])
	}

	fn collider() -> Collider {
		Collider::cuboid(0.5, 0.5, 0.5)
	}
}
