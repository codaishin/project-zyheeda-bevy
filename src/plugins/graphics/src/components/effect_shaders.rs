use super::{camera_labels::SecondPass, insert_recursively::InsertRecursively};
use bevy::{prelude::Component, render::view::RenderLayers};
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq)]
#[require(InsertRecursively::<RenderLayers>(SecondPass::default))]
pub(crate) struct EffectShader<T>(PhantomData<T>);

impl<T> Default for EffectShader<T> {
	fn default() -> Self {
		Self(PhantomData)
	}
}
