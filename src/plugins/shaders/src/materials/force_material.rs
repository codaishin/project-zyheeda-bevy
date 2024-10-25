use crate::traits::shadows_aware_material::ShadowsAwareMaterial;
use bevy::{
	prelude::*,
	render::render_resource::{AsBindGroup, ShaderRef},
};
use common::traits::process_delta::ProcessDelta;
use std::time::Duration;

#[derive(Asset, TypePath, AsBindGroup, Clone, Default)]
pub struct ForceMaterial {
	#[uniform(0)]
	pub color: LinearRgba,
	pub alpha_mode: AlphaMode,
	#[uniform(1)]
	pub(crate) lifetime_secs: f32,
}

impl From<Srgba> for ForceMaterial {
	fn from(color: Srgba) -> Self {
		Self {
			color: color.into(),
			alpha_mode: AlphaMode::Blend,
			lifetime_secs: 0.,
		}
	}
}

impl ProcessDelta for ForceMaterial {
	fn process_delta(&mut self, delta: Duration) {
		self.lifetime_secs += delta.as_secs_f32();
	}
}

impl Material for ForceMaterial {
	fn vertex_shader() -> ShaderRef {
		"shaders/force_shader.wgsl".into()
	}

	fn fragment_shader() -> ShaderRef {
		"shaders/force_shader.wgsl".into()
	}

	fn alpha_mode(&self) -> AlphaMode {
		self.alpha_mode
	}
}

impl ShadowsAwareMaterial for ForceMaterial {
	fn shadows_enabled() -> bool {
		false
	}
}
