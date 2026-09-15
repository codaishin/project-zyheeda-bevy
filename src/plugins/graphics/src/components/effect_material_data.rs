use crate::{
	components::{camera_labels::CompositePass, model_render_layers::ModelRenderLayers},
	materials::effect_material::EffectMaterial,
};
use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq, Clone)]
#[require(Visibility::Hidden, ModelRenderLayers::from(CompositePass))]
pub struct EffectMaterialData {
	pub(crate) first_pass: Handle<Image>,
	pub(crate) base_color: LinearRgba,
	pub(crate) fresnel_color: LinearRgba,
	pub(crate) flags: u32,
	pub(crate) impacts: Vec<Impact>,
	pub(crate) material: Handle<EffectMaterial>,
}

impl EffectMaterialData {
	const DEFAULT_COLOR: Srgba = Srgba {
		red: 1.,
		green: 1.,
		blue: 1.,
		alpha: 0.,
	};
	const DEFAULT_FRESNEL: Srgba = Srgba {
		red: 0.,
		green: 0.,
		blue: 0.,
		alpha: 0.,
	};
}

impl Default for EffectMaterialData {
	fn default() -> Self {
		Self {
			first_pass: Handle::default(),
			base_color: LinearRgba::from(Self::DEFAULT_COLOR),
			fresnel_color: LinearRgba::from(Self::DEFAULT_FRESNEL),
			flags: 0,
			impacts: vec![],
			material: Handle::default(),
		}
	}
}
