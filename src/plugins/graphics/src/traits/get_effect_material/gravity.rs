use crate::{
	materials::gravity_material::GravityMaterial,
	traits::get_effect_material::GetEffectMaterial,
};
use bevy::color::{palettes::css::LIGHT_GRAY, Alpha};
use common::effects::gravity::Gravity;

impl GetEffectMaterial for Gravity {
	type TMaterial = GravityMaterial;

	fn get_effect_material() -> Self::TMaterial {
		GravityMaterial::from(LIGHT_GRAY.with_alpha(0.5))
	}
}
