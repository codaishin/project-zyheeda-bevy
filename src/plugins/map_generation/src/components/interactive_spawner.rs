use crate::components::{map::objects::MapObject, spawner_active::SpawnerActive};
use bevy::prelude::*;
use common::traits::handles_interactive::Interactive;

#[derive(Component, Debug, PartialEq)]
#[require(SpawnerActive, MapObject)]
pub(crate) struct InteractiveSpawner(pub(crate) Interactive);
