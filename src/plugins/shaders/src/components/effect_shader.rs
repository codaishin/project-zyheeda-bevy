pub(crate) mod force_shield;

use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct EffectShader<TEffect>(pub(crate) TEffect);
