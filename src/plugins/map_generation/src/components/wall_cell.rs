use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::components::is_blocker::{Blocker, IsBlocker};
use std::sync::LazyLock;

#[derive(Component, Debug, PartialEq)]
#[require(
	Transform,
	IsBlocker = [Blocker::Physical],
	Collider = UNIT_CUBE.clone(),
)]
pub(crate) struct WallCell;

static UNIT_CUBE: LazyLock<Collider> = LazyLock::new(|| Collider::cuboid(0.5, 0.5, 0.5));
