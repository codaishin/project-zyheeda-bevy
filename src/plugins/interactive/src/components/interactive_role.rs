use bevy::prelude::*;
use common::traits::handles_interactive::Interactive;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct InteractiveRole(pub(crate) Interactive);
