use bevy::{
	pbr::{ExtendedMaterial, MaterialExtension},
	prelude::*,
	render::render_resource::AsBindGroup,
	shader::ShaderRef,
};
use macros::asset_path;

pub(crate) type StandardLitMaterial = ExtendedMaterial<StandardMaterial, LitMaterial>;

#[derive(Asset, AsBindGroup, Reflect, Debug, PartialEq, Clone)]
pub(crate) struct LitMaterial {
	#[uniform(100)]
	pub(crate) player_position: Vec3,
	#[uniform(101)]
	falloff: f32,
	#[uniform(102)]
	min_light: f32,
	#[texture(103, dimension = "cube")]
	#[sampler(104)]
	pub(crate) los: Handle<Image>,
}

impl LitMaterial {
	pub(crate) const SHADER: &str = asset_path!("shaders/lit_shader.wgsl");
	pub(crate) const LINEAR_FALLOFF: f32 = 0.1;
	pub(crate) const MIN_LIGHT: f32 = 0.01;
	pub(crate) const MAX_LIGHT_DISTANCE: f32 = 1. / Self::LINEAR_FALLOFF;

	pub(crate) fn from_player_position(player_position: Vec3) -> Self {
		Self {
			player_position,
			..default()
		}
	}

	pub(crate) fn with_los_cubemap(mut self, los: Handle<Image>) -> Self {
		self.los = los;
		self
	}
}

impl Default for LitMaterial {
	fn default() -> Self {
		Self {
			player_position: Vec3::ZERO,
			falloff: Self::LINEAR_FALLOFF,
			min_light: Self::MIN_LIGHT,
			los: Handle::default(),
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
