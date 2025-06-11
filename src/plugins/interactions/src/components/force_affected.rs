use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct ForceAffected;
