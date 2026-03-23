use bevy::{prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef};

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

	fn enable_shadows() -> bool {
		false
	}
}
