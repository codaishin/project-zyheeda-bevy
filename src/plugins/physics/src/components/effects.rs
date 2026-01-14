pub(crate) mod force;
pub(crate) mod gravity;
pub(crate) mod health_damage;

use bevy::prelude::*;
use common::traits::handles_skill_physics::Effect;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct Effects(pub(crate) Vec<Effect>);
