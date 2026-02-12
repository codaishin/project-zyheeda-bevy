use bevy::prelude::*;
use common::tools::UnitsPerSecond;

#[derive(Component, Debug, PartialEq)]
#[require(Transform)]
pub(crate) struct SetVelocityForward(pub(crate) UnitsPerSecond);
