use bevy::prelude::*;
use macros::{SavableComponent, serde_model};

#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy)]
#[savable_component(id = "is_active")]
#[component(immutable)]
pub(crate) struct IsActive;
