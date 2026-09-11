use crate::{
	components::effect_material_data::{EffectFlag, EffectMaterialData},
	traits::modify_material::ModifyMaterial,
};
use common::prelude::*;

impl ModifyMaterial for Gravity {
	fn modify_material(skill_material: &mut EffectMaterialData) {
		skill_material.add_flag(EffectFlag::Distortion);
	}
}
