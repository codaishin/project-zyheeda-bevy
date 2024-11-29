use super::EffectShader;
use crate::{
	materials::force_material::ForceMaterial,
	traits::get_effect_material::GetEffectMaterial,
};
use bevy::color::palettes::css::LIGHT_CYAN;
use common::effects::force_shield::ForceShield;

impl GetEffectMaterial for EffectShader<ForceShield> {
	type TMaterial = ForceMaterial;

	fn get_effect_material(&self) -> Self::TMaterial {
		ForceMaterial::from(LIGHT_CYAN * 1.5)
	}
}
