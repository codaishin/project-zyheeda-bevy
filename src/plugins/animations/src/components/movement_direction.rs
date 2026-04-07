use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct MovementDirection(pub(crate) Option<Dir3>);
