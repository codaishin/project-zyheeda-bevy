use crate::{
	components::{camera_labels::CompositePass, model_render_layers::ModelRenderLayers},
	materials::effect_material::EffectMaterial,
};
use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq, Default, Clone)]
#[require(Visibility::Hidden, ModelRenderLayers::from(CompositePass))]
pub struct EffectMaterialData {
	pub(crate) impacts: Vec<Impact>,
	pub(crate) material: Handle<EffectMaterial>,
}

impl View<Handle<EffectMaterial>> for EffectMaterialData {
	fn view(&self) -> &'_ Handle<EffectMaterial> {
		&self.material
	}
}
