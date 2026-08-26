use bevy::prelude::*;
use common::prelude::*;
use macros::{SavableComponent, serde_model};

#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[savable_component(id = "set_face_override")]
#[require(CanFace)]
pub struct SetFaceOverride(pub Face);

#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[savable_component(id = "set_face")]
#[require(CanFace)]
pub struct SetFace(pub Face);

#[derive(Component, Debug, PartialEq, Default)]
pub struct CanFace;
