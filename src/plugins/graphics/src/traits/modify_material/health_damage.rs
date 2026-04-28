use crate::{
	materials::effect_material::{EffectFlag, EffectMaterial},
	traits::modify_material::ModifyMaterial,
};
use bevy::color::palettes::css::WHITE;
use common::effects::health_damage::HealthDamage;

impl ModifyMaterial for HealthDamage {
	fn modify_material(material: &mut EffectMaterial) {
		material.add_flag(EffectFlag::base_color(WHITE * 10.0));
	}
}
