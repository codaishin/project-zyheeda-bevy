use crate::{
	components::{camera_labels::SecondPass, pass_layer::PassLayer},
	materials::effect_material::EffectMaterial,
};
use bevy::prelude::*;

#[derive(Component, Default)]
#[require(Visibility::Hidden, PassLayer::from(SecondPass))]
pub struct EffectMaterialHandle {
	pub(crate) material: Handle<EffectMaterial>,
}
