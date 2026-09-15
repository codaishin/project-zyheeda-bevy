use crate::{
	components::{camera_labels::CompositePass, model_render_layers::ModelRenderLayers},
	materials::effect_material::EffectMaterial,
};
use bevy::prelude::*;
use common::prelude::*;
use zyheeda_core::{math::f32_not_nan::F32FiniteStrictlyPositive, new_f32};

#[derive(Component, Debug, PartialEq, Default, Clone)]
#[require(
	GlobalTransform,
	Visibility::Hidden,
	ModelRenderLayers::from(CompositePass)
)]
pub struct EffectMaterialData {
	pub(crate) impacts: Vec<Impact>,
	pub(crate) material: Handle<EffectMaterial>,
}

impl View<Handle<EffectMaterial>> for EffectMaterialData {
	fn view(&self) -> &'_ Handle<EffectMaterial> {
		&self.material
	}
}

pub(crate) type ImpactStrength = F32FiniteStrictlyPositive;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Impact {
	pub(crate) local: Vec3,
	pub(crate) strength: ImpactStrength,
}

impl Impact {
	pub(crate) fn from_local(position: Vec3) -> Self {
		Self {
			local: position,
			strength: new_f32!(ImpactStrength(1.)),
		}
	}
}
