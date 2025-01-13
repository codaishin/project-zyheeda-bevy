use crate::traits::shadows_aware_material::ShadowsAwareMaterial;
use bevy::{
	prelude::*,
	render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, TypePath, AsBindGroup, Clone, Default)]
pub struct GravityMaterial {
	#[texture(0)]
	#[sampler(1)]
	pub first_pass: Handle<Image>,
	pub alpha_mode: AlphaMode,
}

impl From<Handle<Image>> for GravityMaterial {
	fn from(first_pass: Handle<Image>) -> Self {
		Self {
			first_pass,
			alpha_mode: AlphaMode::Opaque,
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
