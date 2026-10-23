use bevy::{
	prelude::*,
	render::{RenderApp, extract_component::ExtractComponent},
};

#[derive(Component, ExtractComponent, Debug, PartialEq, Clone)]
#[extract_app(RenderApp)]
pub(crate) struct DistanceTexture(pub(crate) Handle<Image>);
