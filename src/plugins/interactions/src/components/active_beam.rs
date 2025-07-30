use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(Transform, Visibility)]
pub(crate) struct ActiveBeam {
	pub(crate) source: Vec3,
	pub(crate) target: Vec3,
}
