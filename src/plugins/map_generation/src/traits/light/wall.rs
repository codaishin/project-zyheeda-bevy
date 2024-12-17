use crate::{
	components::{Light, Wall},
	traits::ExtraComponentsDefinition,
};
use bevy::prelude::*;
use common::{
	errors::Error,
	tools::{Intensity, IntensityChangePerSecond, Units},
	traits::{
		cache::GetOrCreateTypeAsset,
		clamp_zero_positive::ClampZeroPositive,
		handles_lights::{HandlesLights, Responsive},
		prefab::{GetOrCreateAssets, Prefab},
	},
};

impl ExtraComponentsDefinition for Light<Wall> {
	fn target_names() -> Vec<String> {
		vec![
			"LightNZData".to_owned(),
			"LightNXData".to_owned(),
			"LightPZData".to_owned(),
			"LightPXData".to_owned(),
		]
	}

	fn insert_bundle(entity: &mut EntityCommands) {
		entity.try_insert(Light::<Wall>::default());
	}
}

struct WallLightOn;

struct WallLightOff;

impl<TLights> Prefab<TLights> for Light<Wall>
where
	TLights: HandlesLights,
{
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		mut assets: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		let model = entity.id();
		let light_on_material = assets.get_or_create_for::<WallLightOn>(|| StandardMaterial {
			base_color: Color::WHITE,
			emissive: Color::linear_rgb(140.0, 140.0, 140.0).into(),
			..default()
		});
		let light_off_material = assets.get_or_create_for::<WallLightOff>(|| StandardMaterial {
			base_color: Color::BLACK,
			..default()
		});

		entity.with_children(|parent| {
			let light = parent
				.spawn((
					PointLight {
						shadows_enabled: false,
						intensity: 0.,
						..default()
					},
					Visibility::Hidden,
				))
				.id();
			parent.spawn(TLights::responsive_light_bundle(Responsive {
				model,
				light,
				range: Units::new(3.5),
				light_on_material,
				light_off_material: light_off_material.clone(),
				max: Intensity::new(8000.),
				change: IntensityChangePerSecond::new(4000.),
			}));
		});

		entity.try_insert(MeshMaterial3d(light_off_material));

		Ok(())
	}
}
