use bevy::{
	prelude::*,
	render::{extract_component::ExtractComponent, render_resource::ShaderType},
};
use macros::asset_path;

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
pub(crate) struct PostProcessCamera {
	pub(crate) outline_color: LinearRgba,
	pub(crate) see_through_color: LinearRgba,
}

impl PostProcessCamera {
	pub(crate) const SHADER_PATH: &str = asset_path!("shaders/post_processing.wgsl");
}
