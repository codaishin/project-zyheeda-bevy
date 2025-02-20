use crate::{
	components::{camera_labels::SecondPass, effect_shaders::EffectShader},
	traits::get_effect_material::GetEffectMaterial,
};
use bevy::{
	color::palettes::css::WHITE,
	ecs::system::EntityCommands,
	prelude::*,
	render::view::RenderLayers,
};
use common::{effects::deal_damage::DealDamage, errors::Error, traits::prefab::Prefab};

impl GetEffectMaterial for DealDamage {
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

impl Prefab<()> for EffectShader<DealDamage> {
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
	) -> Result<(), Error> {
		entity.with_children(|parent| {
			parent.spawn((
				RenderLayers::from(SecondPass),
				PointLight {
					color: Color::from(WHITE),
					intensity: 8000.,
					shadows_enabled: true,
					..default()
				},
			));
		});

		Ok(())
	}
}
