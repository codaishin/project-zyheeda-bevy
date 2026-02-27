use bevy::prelude::*;
use common::traits::handles_map_generation::AgentType;

#[derive(Component, Debug, PartialEq)]
#[require(SpawnerActive)]
pub(crate) struct AgentSpawner(pub(crate) AgentType);

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct SpawnerActive;
