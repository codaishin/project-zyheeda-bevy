use super::GetEffectMaterial;
use crate::materials::force_material::ForceMaterial;
use bevy::color::palettes::css::LIGHT_CYAN;
use interactions::components::force::Force;

impl GetEffectMaterial for Force {
	type TMaterial = ForceMaterial;

	fn get_effect_material(&self) -> Self::TMaterial {
		ForceMaterial::from(LIGHT_CYAN)
	}
}
