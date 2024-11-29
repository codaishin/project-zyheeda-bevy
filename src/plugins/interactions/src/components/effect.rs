pub(crate) mod deal_damage;
pub(crate) mod gravity;

use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct Effect<TEffect>(pub(crate) TEffect);
