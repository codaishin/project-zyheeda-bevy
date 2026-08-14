use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(Transform)]
pub(crate) struct SetVelocityForward(pub(crate) UnitsPerSecond);
