pub(crate) mod deal_damage;
pub(crate) mod force_shield;
pub(crate) mod gravity;

use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub struct Effect<TEffect>(pub(crate) TEffect);
