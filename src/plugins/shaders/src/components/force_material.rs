use bevy::{
	prelude::*,
	render::render_resource::{AsBindGroup, ShaderRef},
};

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
