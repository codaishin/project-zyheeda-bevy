use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct MovementDirection(pub(crate) Dir3);
