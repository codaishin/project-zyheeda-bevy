use super::GetEffectMaterial;
use crate::materials::gravity_material::GravityMaterial;
use bevy::color::{palettes::css::LIGHT_GRAY, Alpha};
use interactions::components::gravity::Gravity;

impl GetEffectMaterial for Gravity {
	type TMaterial = GravityMaterial;

	fn get_effect_material(&self) -> Self::TMaterial {
		GravityMaterial::from(LIGHT_GRAY.with_alpha(0.5))
	}
}
