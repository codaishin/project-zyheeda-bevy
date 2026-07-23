use bevy::{
	pbr::{ExtendedMaterial, MaterialExtension},
	prelude::*,
	render::render_resource::AsBindGroup,
	shader::ShaderRef,
};
use macros::asset_path;

pub(crate) type StandardLitMaterial = ExtendedMaterial<StandardMaterial, LitMaterial>;

#[derive(Asset, AsBindGroup, Reflect, Debug, PartialEq, Clone, Copy)]
pub(crate) struct LitMaterial {
	#[uniform(100)]
	pub(crate) player_position: Vec3,
	#[uniform(101)]
	pub(crate) falloff: f32,
	#[uniform(102)]
	pub(crate) min_light: f32,
}

impl LitMaterial {
	const SHADER: &str = asset_path!("shaders/lit_shader.wgsl");
	const DEFAULT_FALLOFF: f32 = 0.1;
	const DEFAULT_MIN_LIGHT: f32 = 0.01;

	pub(crate) fn from_player_position(player_position: Vec3) -> Self {
		Self {
			player_position,
			..default()
		}
	}
}

impl Default for LitMaterial {
	fn default() -> Self {
		Self {
			player_position: Vec3::ZERO,
			falloff: Self::DEFAULT_FALLOFF,
			min_light: Self::DEFAULT_MIN_LIGHT,
		}
	}
}

impl MaterialExtension for LitMaterial {
	fn fragment_shader() -> ShaderRef {
		Self::SHADER.into()
	}

	fn deferred_fragment_shader() -> ShaderRef {
		Self::SHADER.into()
	}
}
