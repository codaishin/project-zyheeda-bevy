use super::GetEffectMaterial;
use crate::materials::gravity_material::GravityMaterial;
use bevy::color::palettes::css::LIGHT_GRAY;
use interactions::components::gravity::Gravity;

impl GetEffectMaterial for Gravity {
	type TMaterial = GravityMaterial;

	fn get_effect_material(&self) -> Self::TMaterial {
		GravityMaterial::from(LIGHT_GRAY)
	}
}
