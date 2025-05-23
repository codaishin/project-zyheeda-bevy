use crate::components::camera_labels::SecondPass;
use bevy::{color::palettes::css::WHITE, prelude::*, render::view::RenderLayers};
use common::{errors::Error, traits::prefab::Prefab};

#[derive(Component, Debug, PartialEq, Default)]
pub struct DamageEffectShaders;

impl Prefab<()> for DamageEffectShaders {
	fn insert_prefab_components(&self, entity: &mut EntityCommands) -> Result<(), Error> {
		entity.with_child((
			RenderLayers::from(SecondPass),
			PointLight {
				color: Color::from(WHITE),
				intensity: 8000.,
				shadows_enabled: true,
				..default()
			},
		));

		Ok(())
	}
}
