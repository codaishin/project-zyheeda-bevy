use bevy::prelude::*;
use common::tools::Units;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct CenterOffset(pub(crate) Units);
