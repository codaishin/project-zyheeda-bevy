pub(crate) mod damage_effect_shaders;

use bevy::prelude::Component;
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct EffectShader<T>(PhantomData<T>);

impl<T> Default for EffectShader<T> {
	fn default() -> Self {
		Self(PhantomData)
	}
}
