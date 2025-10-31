use bevy::prelude::*;
use common::traits::handles_orientation::FaceTargetIs;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct FaceTarget(pub(crate) FaceTargetIs);
