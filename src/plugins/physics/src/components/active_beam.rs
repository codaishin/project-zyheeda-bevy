use bevy::prelude::*;
use common::tools::Units;

#[derive(Component, Debug, PartialEq)]
#[require(Transform, Visibility)]
pub(crate) struct ActiveBeam {
	pub(crate) length: Units,
}
