use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct CurrentMovementDirection(pub(crate) Option<Dir3>);
