use bevy::prelude::*;
use common::tools::Units;

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub(crate) struct TorchLight {
	pub(crate) intensity: Units,
}
