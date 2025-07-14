use bevy::prelude::*;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Lifetime(pub(crate) Duration);
