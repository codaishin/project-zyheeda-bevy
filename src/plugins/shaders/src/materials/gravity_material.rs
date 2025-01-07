use crate::traits::shadows_aware_material::ShadowsAwareMaterial;
use bevy::{
	prelude::*,
	render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, TypePath, AsBindGroup, Clone, Default)]
pub struct GravityMaterial {
	#[uniform(0)]
	pub color: LinearRgba,
	pub alpha_mode: AlphaMode,
}

impl From<Srgba> for GravityMaterial {
	fn from(color: Srgba) -> Self {
		Self {
			color: color.into(),
			alpha_mode: AlphaMode::Blend,
		}
	}
}

impl Material for GravityMaterial {
	fn fragment_shader() -> ShaderRef {
		"shaders/gravity_shader.wgsl".into()
	}

	fn alpha_mode(&self) -> AlphaMode {
		self.alpha_mode
	}
}

impl ShadowsAwareMaterial for GravityMaterial {
	fn shadows_enabled() -> bool {
		false
	}
}
