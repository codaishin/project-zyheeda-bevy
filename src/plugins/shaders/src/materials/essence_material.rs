use bevy::{
	prelude::*,
	render::render_resource::{AsBindGroup, ShaderRef},
};

use crate::traits::shadows_aware_material::ShadowsAwareMaterial;

#[derive(Asset, TypePath, AsBindGroup, Clone, Default)]
pub struct EssenceMaterial {
	#[uniform(0)]
	pub texture_color: LinearRgba,
	#[uniform(1)]
	pub fill_color: LinearRgba,
	#[uniform(2)]
	pub shine: LinearRgba,
	#[texture(3)]
	#[sampler(4)]
	pub color_texture: Option<Handle<Image>>,
	pub alpha_mode: AlphaMode,
}

impl Material for EssenceMaterial {
	fn vertex_shader() -> ShaderRef {
		"shaders/essence_shader.wgsl".into()
	}

	fn fragment_shader() -> ShaderRef {
		"shaders/essence_shader.wgsl".into()
	}

	fn alpha_mode(&self) -> AlphaMode {
		self.alpha_mode
	}
}

impl ShadowsAwareMaterial for EssenceMaterial {
	fn casts_shadows() -> bool {
		true
	}
}
