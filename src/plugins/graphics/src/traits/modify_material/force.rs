use crate::{
	materials::effect_material::{EffectFlag, EffectMaterial},
	traits::modify_material::ModifyMaterial,
};
use bevy::color::palettes::css::LIGHT_CYAN;
use common::effects::force::Force;

impl ModifyMaterial for Force {
	fn modify_material(material: &mut EffectMaterial) {
		material.add_flag(EffectFlag::fresnel(LIGHT_CYAN * 1.5));
	}
}
