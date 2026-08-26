use bevy::prelude::*;
use macros::{SavableComponent, serde_model};

#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[component(immutable)]
#[savable_component(id = "agents loaded marker")]
pub(crate) struct AgentsLoaded;
