use crate::components::map::objects::MapObject;
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Default)]
#[require(MapObject)]
pub(crate) struct NavMesh;
