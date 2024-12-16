pub(crate) mod deal_damage;
pub(crate) mod force_shield;
pub(crate) mod gravity;

use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct EffectShader<TEffect>(pub(crate) PhantomData<TEffect>);

impl<T> Default for EffectShader<T> {
	fn default() -> Self {
		Self(PhantomData)
	}
}
