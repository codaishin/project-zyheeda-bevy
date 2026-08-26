use bevy::prelude::*;
use macros::{SavableComponent, serde_model};

#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[component(immutable)]
#[savable_component(id = "agents_loaded_marker")]
pub(crate) struct AgentsLoaded;
