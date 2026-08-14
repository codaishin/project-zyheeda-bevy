use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct Attacking {
	pub(crate) has_los: bool,
	pub(crate) player: PersistentEntity,
}
