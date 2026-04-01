use bevy::prelude::*;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[savable_component(id = "movement")]
pub(crate) enum Movement {
	None,
	Direction(Dir3),
	Target(Dir3),
	Path(VecDeque<Vec3>),
}
