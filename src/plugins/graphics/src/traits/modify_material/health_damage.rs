use crate::{
	materials::effect_material::{EffectFlag, EffectMaterial},
	traits::modify_material::ModifyMaterial,
};
use bevy::color::palettes::css::WHITE;
use common::prelude::*;

impl ModifyMaterial for HealthDamage {
	fn modify_material(material: &mut EffectMaterial) {
		material.add_flag(EffectFlag::base_color(WHITE * 10.0));
	}
}
