use crate::{
	materials::force_material::ForceMaterial,
	traits::get_effect_material::GetEffectMaterial,
};
use bevy::{color::palettes::css::LIGHT_CYAN, prelude::*};
use common::effects::force_shield::Force;

impl GetEffectMaterial for Force {
	type TMaterial = ForceMaterial;

	fn get_effect_material(_: &Handle<Image>) -> Self::TMaterial {
		ForceMaterial::from(LIGHT_CYAN * 1.5)
	}
}
