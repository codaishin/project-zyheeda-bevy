pub(crate) mod level;

use bevy::prelude::*;

pub(crate) struct Wall;

pub(crate) struct WallBack;

pub(crate) struct Corridor;

impl Corridor {
	pub const MODEL_PATH_PREFIX: &'static str = "models/corridor_";
}

#[derive(Component)]
pub(crate) struct Unlit;
