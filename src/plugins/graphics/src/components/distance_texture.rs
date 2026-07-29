use bevy::{prelude::*, render::extract_component::ExtractComponent};

#[derive(Component, ExtractComponent, Debug, PartialEq, Clone)]
pub(crate) struct DistanceTexture(pub(crate) Handle<Image>);
