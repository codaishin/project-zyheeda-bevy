use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct CurrentForwardPitch(pub(crate) Option<DirForwardPitch>);
