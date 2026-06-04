use bevy::{
	camera::visibility::{Layer, RenderLayers},
	prelude::*,
};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct PassLayers {
	layer: Layer,
}

impl From<Layer> for PassLayers {
	fn from(layer: Layer) -> Self {
		Self { layer }
	}
}

impl From<&PassLayers> for RenderLayers {
	fn from(PassLayers { layer }: &PassLayers) -> Self {
		RenderLayers::layer(*layer)
	}
}
