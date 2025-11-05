use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct Chasing {
	pub(crate) player: Entity,
}
