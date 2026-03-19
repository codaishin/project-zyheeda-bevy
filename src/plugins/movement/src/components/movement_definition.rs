use bevy::prelude::*;
use common::tools::{Units, UnitsPerSecond};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[savable_component(id = "movement definition")]
pub struct MovementDefinition {
	pub(crate) radius: Units,
	pub(crate) speed: UnitsPerSecond,
}
