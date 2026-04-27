use crate::{
	materials::effect_material::{EffectFlag, EffectMaterial},
	traits::modify_material::ModifyMaterial,
};
use common::effects::gravity::Gravity;

impl ModifyMaterial for Gravity {
	fn modify_material(skill_material: &mut EffectMaterial) {
		skill_material.add_flag(EffectFlag::Distortion);
	}
}
