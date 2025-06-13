use bevy::prelude::*;
use common::traits::handles_saving::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct ForceAffected;

impl SavableComponent for ForceAffected {
	type TDto = Self;
}
