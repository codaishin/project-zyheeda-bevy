use super::EffectShader;
use crate::traits::get_effect_material::GetEffectMaterial;
use bevy::{color::palettes::css::WHITE, ecs::system::EntityCommands, prelude::*};
use common::{
	effects::deal_damage::DealDamage,
	errors::Error,
	traits::prefab::{GetOrCreateAssets, Prefab},
};

impl GetEffectMaterial for EffectShader<DealDamage> {
	type TMaterial = StandardMaterial;

	fn get_effect_material(&self) -> Self::TMaterial {
		let base_color = Color::from(WHITE);
		let emissive_amount = 2300.0;

		StandardMaterial {
			emissive: base_color.to_linear() * emissive_amount,
			base_color,
			..default()
		}
	}
}

impl Prefab<()> for EffectShader<DealDamage> {
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		_: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		entity.try_insert(PointLightBundle {
			point_light: PointLight {
				color: Color::from(WHITE),
				intensity: 8000.,
				shadows_enabled: true,
				..default()
			},
			..default()
		});

		Ok(())
	}
}
