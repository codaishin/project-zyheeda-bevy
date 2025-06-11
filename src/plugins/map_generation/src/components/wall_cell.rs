use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	blocker::{Blocker, Blockers},
	components::no_target::NoTarget,
};
use std::sync::LazyLock;

#[derive(Component, Debug, PartialEq)]
#[require(
	Transform,
	Blockers = [Blocker::Physical],
	Collider = UNIT_CUBE.clone(),
	NoTarget
)]
pub(crate) struct WallCell;

static UNIT_CUBE: LazyLock<Collider> = LazyLock::new(|| Collider::cuboid(0.5, 0.5, 0.5));
