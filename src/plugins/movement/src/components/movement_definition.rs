use bevy::prelude::*;
use common::tools::{Units, UnitsPerSecond};

#[derive(Component, Debug, PartialEq, Default)]
pub struct MovementDefinition {
	pub(crate) radius: Units,
	pub(crate) speed: UnitsPerSecond,
}
