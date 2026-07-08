use crate::components::map::MapObjectSource;
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct SpawnedFrom(pub(crate) MapObjectSource);
