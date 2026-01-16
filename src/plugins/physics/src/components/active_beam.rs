use bevy::prelude::*;
use common::tools::Units;

#[derive(Component, Debug, PartialEq)]
#[require(Transform, Visibility)]
pub(crate) struct ActiveBeam {
	pub(crate) length: Units,
}

impl Default for ActiveBeam {
	fn default() -> Self {
		Self {
			length: Units::EPSILON,
		}
	}
}
