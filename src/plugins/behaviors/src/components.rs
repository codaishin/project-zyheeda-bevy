pub(crate) mod attacking;
pub(crate) mod fix_points;
pub(crate) mod ground_target;
pub(crate) mod movement;
pub(crate) mod set_to_move_forward;
pub(crate) mod skill_behavior;
pub(crate) mod skill_usage;
pub(crate) mod when_traveled_insert;

use bevy::prelude::*;
use common::{components::persistent_entity::PersistentEntity, traits::handles_orientation::Face};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Always;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Once;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct OverrideFace(pub Face);

#[derive(Component, Debug, PartialEq)]
pub struct SetFace(pub Face);

#[derive(Component, Debug, PartialEq)]
pub struct Chase(pub PersistentEntity);

#[derive(Component, Debug, PartialEq)]
pub struct Attack(pub PersistentEntity);
