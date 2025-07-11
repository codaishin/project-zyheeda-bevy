use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Default, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[component(immutable)]
pub enum Essence {
	#[default]
	None,
	Force,
}
