use bevy::prelude::*;
use macros::{SavableComponent, serde_model};
use std::time::Duration;

#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone)]
#[savable_component(id = "lifetime")]
pub struct Lifetime(pub(crate) Duration);
