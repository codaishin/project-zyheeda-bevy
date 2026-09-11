use crate::{
	components::effect_material_data::{EffectFlag, EffectMaterialData},
	traits::modify_material::ModifyMaterial,
};
use bevy::color::palettes::css::WHITE;
use common::prelude::*;

impl ModifyMaterial for HealthDamage {
	fn modify_material(material: &mut EffectMaterialData) {
		material.add_flag(EffectFlag::base_color(WHITE * 10.0));
	}
}
