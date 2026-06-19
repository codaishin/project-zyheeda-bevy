use bevy::{
	prelude::*,
	render::{extract_component::ExtractComponent, render_resource::ShaderType},
};
use common::tools::pixel::Pixel;
use macros::asset_path;

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
pub(crate) struct PostProcessCamera {
	outline_color: LinearRgba,
	outline_width: f32,
	see_through_color: LinearRgba,
	dark_region_light_factor: f32,
}

impl PostProcessCamera {
	pub(crate) const SHADER_PATH: &str = asset_path!("shaders/post_processing.wgsl");

	pub(crate) fn new(args: PostProcessArgs) -> Self {
		Self {
			outline_color: args.outline_color.into(),
			outline_width: args.outline_width.0,
			see_through_color: args.see_through_color.into(),
			dark_region_light_factor: args.dark_region_light_factor,
		}
	}
}

pub(crate) struct PostProcessArgs {
	pub(crate) outline_color: Srgba,
	pub(crate) outline_width: Pixel,
	pub(crate) see_through_color: Srgba,
	pub(crate) dark_region_light_factor: f32,
}
