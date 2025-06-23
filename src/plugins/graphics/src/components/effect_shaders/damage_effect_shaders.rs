use crate::components::camera_labels::SecondPass;
use bevy::{color::palettes::css::WHITE, prelude::*, render::view::RenderLayers};
use common::{
	errors::Error,
	traits::{
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};

#[derive(Component, Debug, PartialEq, Default)]
pub struct DamageEffectShaders;

impl Prefab<()> for DamageEffectShaders {
	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Error> {
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
