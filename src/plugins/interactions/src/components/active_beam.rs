use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(Transform, Visibility)]
#[component(immutable)]
pub(crate) struct ActiveBeam;
