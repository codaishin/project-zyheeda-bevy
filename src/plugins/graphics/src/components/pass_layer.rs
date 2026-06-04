use bevy::{camera::visibility::Layer, prelude::*};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct PassLayer {
	pub(crate) layer: Layer,
}
