use bevy::{camera::RenderTarget, core_pipeline::prepass::DepthPrepass, prelude::*};

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
#[require(DepthPrepass, RenderTarget::None { size: UVec2 { x: 1, y: 1 }})]
pub(crate) struct OnlyDepthPrepass;
