use crate::traits::get_effect_material::GetEffectMaterial;
use bevy::{color::palettes::css::WHITE, prelude::*};
use common::effects::health_damage::HealthDamage;

impl GetEffectMaterial for HealthDamage {
	type TMaterial = StandardMaterial;

	fn get_effect_material(_: &Handle<Image>) -> Self::TMaterial {
		let base_color = Color::from(WHITE);
		let emissive_amount = 2300.0;

		StandardMaterial {
			emissive: base_color.to_linear() * emissive_amount,
			base_color,
			..default()
		}
	}
}
