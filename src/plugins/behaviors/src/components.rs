pub(crate) mod attacking;
pub(crate) mod face_target;
pub(crate) mod fix_points;
pub(crate) mod ground_target;
pub(crate) mod movement;
pub(crate) mod movement_definition;
pub(crate) mod set_motion_forward;
pub(crate) mod skill_behavior;
pub(crate) mod skill_usage;
pub(crate) mod when_traveled_insert;

use bevy::prelude::*;
use common::traits::handles_orientation::Face;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Always;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Once;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[require(CanFace)]
pub struct SetFaceOverride(pub Face);

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[require(CanFace)]
pub struct SetFace(pub Face);

#[derive(Component, Debug, PartialEq, Default)]
pub struct CanFace;
