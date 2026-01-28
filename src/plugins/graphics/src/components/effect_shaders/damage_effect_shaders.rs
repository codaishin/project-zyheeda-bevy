use crate::components::camera_labels::SecondPass;
use bevy::{color::palettes::css::WHITE, prelude::*, render::view::RenderLayers};
use common::{
	errors::Unreachable,
	traits::{
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
pub struct DamageEffectShaders;

impl Prefab<()> for DamageEffectShaders {
	type TError = Unreachable;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Unreachable> {
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
