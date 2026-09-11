use bevy::{prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef};

use crate::components::effect_material_data::EffectMaterialData;

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
}

impl From<EffectMaterialData> for EffectMaterial {
	fn from(
		EffectMaterialData {
			first_pass,
			base_color,
			fresnel_color,
			flags,
		}: EffectMaterialData,
	) -> Self {
		Self {
			first_pass,
			base_color,
			fresnel_color,
			flags,
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
