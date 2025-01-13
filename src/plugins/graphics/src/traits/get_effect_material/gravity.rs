use crate::{
	materials::gravity_material::GravityMaterial,
	traits::get_effect_material::GetEffectMaterial,
};
use bevy::{image::Image, prelude::*};
use common::effects::gravity::Gravity;

impl GetEffectMaterial for Gravity {
	type TMaterial = GravityMaterial;

	fn get_effect_material(first_pass: &Handle<Image>) -> Self::TMaterial {
		GravityMaterial::from(first_pass.clone())
	}
}
