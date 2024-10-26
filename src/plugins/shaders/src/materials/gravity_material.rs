use crate::traits::shadows_aware_material::ShadowsAwareMaterial;
use bevy::{
	prelude::*,
	render::render_resource::{AsBindGroup, ShaderRef},
};
use common::traits::process_delta::ProcessDelta;
use std::time::Duration;

#[derive(Asset, TypePath, AsBindGroup, Clone, Default)]
pub struct GravityMaterial {
	#[uniform(0)]
	pub color: LinearRgba,
	pub alpha_mode: AlphaMode,
	#[uniform(1)]
	pub(crate) lifetime_secs: f32,
}

impl From<Srgba> for GravityMaterial {
	fn from(color: Srgba) -> Self {
		Self {
			color: color.into(),
			alpha_mode: AlphaMode::Blend,
			lifetime_secs: 0.,
		}
	}
}

impl ProcessDelta for GravityMaterial {
	fn process_delta(&mut self, delta: Duration) {
		self.lifetime_secs += delta.as_secs_f32();
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
