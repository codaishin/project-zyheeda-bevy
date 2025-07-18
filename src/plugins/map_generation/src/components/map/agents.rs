use bevy::prelude::*;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub(crate) struct Player;

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub(crate) struct Enemy;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[component(immutable)]
pub(crate) struct AgentsLoaded;
