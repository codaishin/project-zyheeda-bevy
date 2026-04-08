use bevy::prelude::*;
use common::traits::handles_animations::DirForwardPitch;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct CurrentForwardPitch(pub(crate) Option<DirForwardPitch>);
