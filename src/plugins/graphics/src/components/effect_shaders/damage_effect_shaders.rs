use crate::components::camera_labels::SecondPass;
use bevy::{
	camera::visibility::RenderLayers,
	color::palettes::css::WHITE,
	ecs::system::StaticSystemParam,
	prelude::*,
};
use common::{
	errors::Unreachable,
	traits::prefab::{Prefab, PrefabEntityCommands},
};

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
pub struct DamageEffectShaders;

impl Prefab<()> for DamageEffectShaders {
	type TError = Unreachable;
	type TSystemParam<'w, 's> = ();

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: StaticSystemParam<()>,
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
