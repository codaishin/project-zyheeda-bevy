use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct Attacking {
	pub(crate) has_los: bool,
	pub(crate) player: Entity,
}
