use bevy::prelude::*;
use common::traits::handles_orientation::Face;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[require(CanFace)]
pub struct SetFaceOverride(pub Face);

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[require(CanFace)]
pub struct SetFace(pub Face);

#[derive(Component, Debug, PartialEq, Default)]
pub struct CanFace;
