use bevy::{
	pbr::{ExtendedMaterial, MaterialExtension},
	prelude::*,
	render::render_resource::AsBindGroup,
	shader::ShaderRef,
};
use macros::asset_path;

pub(crate) type StandardLitMaterial = ExtendedMaterial<StandardMaterial, LitMaterial>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub(crate) struct LitMaterial {
	#[uniform(100)]
	pub(crate) player_position: Vec3,
}

impl LitMaterial {
	const SHADER: &str = asset_path!("shaders/lit_shader.wgsl");
}

impl MaterialExtension for LitMaterial {
	fn fragment_shader() -> ShaderRef {
		Self::SHADER.into()
	}

	fn deferred_fragment_shader() -> ShaderRef {
		Self::SHADER.into()
	}
}
