pub(crate) mod deal_damage;

use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct Effect<TEffect>(pub(crate) TEffect);
