use crate::{
	components::{camera_labels::CompositePass, model_render_layers::ModelRenderLayers},
	materials::effect_material::EffectMaterial,
};
use bevy::prelude::*;

#[derive(Component, Default)]
#[require(Visibility::Hidden, ModelRenderLayers::from(CompositePass))]
pub struct EffectMaterialHandle {
	pub(crate) material: Handle<EffectMaterial>,
}
