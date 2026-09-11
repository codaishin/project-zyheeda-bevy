use crate::{
	components::effect_material_data::{EffectFlag, EffectMaterialData},
	traits::modify_material::ModifyMaterial,
};
use bevy::color::palettes::css::LIGHT_CYAN;
use common::prelude::*;

impl ModifyMaterial for Force {
	fn modify_material(material: &mut EffectMaterialData) {
		material.add_flag(EffectFlag::fresnel(LIGHT_CYAN * 1.5));
	}
}
