use bevy::prelude::*;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(
	Component,
	SavableComponent,
	Debug,
	PartialEq,
	Eq,
	Hash,
	Default,
	Clone,
	Copy,
	Serialize,
	Deserialize,
)]
#[savable_component(id = "player camera")]
pub struct PlayerCamera;
