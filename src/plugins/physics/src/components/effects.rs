pub(crate) mod force;
pub(crate) mod gravity;
pub(crate) mod health_damage;

use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq, Clone)]
pub(crate) struct Effects(pub(crate) Vec<SkillEffect>);
