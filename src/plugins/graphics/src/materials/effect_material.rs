use crate::components::effect_material_data::EffectMaterialData;
use bevy::{
	prelude::*,
	render::{
		render_resource::{AsBindGroup, ShaderType},
		storage::ShaderBuffer,
	},
	shader::ShaderRef,
};
use std::sync::LazyLock;

pub static NO_IMPACTS: LazyLock<Handle<ShaderBuffer>> = LazyLock::new(Handle::default);

#[derive(Asset, TypePath, AsBindGroup, Debug, PartialEq, Clone)]
pub(crate) struct EffectMaterial {
	#[texture(0)]
	#[sampler(1)]
	first_pass: Handle<Image>,
	#[uniform(2)]
	base_color: LinearRgba,
	#[uniform(3)]
	fresnel_color: LinearRgba,
	#[uniform(4)]
	flags: u32,
	#[storage(5, read_only)]
	impacts: Handle<ShaderBuffer>,
}

impl From<EffectMaterialData> for EffectMaterial {
	fn from(
		EffectMaterialData {
			first_pass,
			base_color,
			fresnel_color,
			flags,
			..
		}: EffectMaterialData,
	) -> Self {
		Self {
			first_pass,
			base_color,
			fresnel_color,
			flags,
			impacts: NO_IMPACTS.clone(),
		}
	}
}

impl Material for EffectMaterial {
	fn fragment_shader() -> ShaderRef {
		"shaders/effect_shader.wgsl".into()
	}

	fn alpha_mode(&self) -> AlphaMode {
		AlphaMode::Blend
	}

	fn enable_shadows() -> bool {
		false
	}
}

#[derive(Debug, PartialEq, ShaderType)]
pub(crate) struct LocalImpact {
	local: Vec3,
	strength: f32,
}
