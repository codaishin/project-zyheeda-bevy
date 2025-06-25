use bevy::prelude::*;
use common::impl_savable_self_non_priority;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct ForceAffected;

impl_savable_self_non_priority!(ForceAffected);
