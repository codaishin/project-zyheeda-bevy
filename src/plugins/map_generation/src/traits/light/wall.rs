use crate::traits::ExtraComponentsDefinition;
use bevy::prelude::*;
use common::{
	tools::{Intensity, IntensityChangePerSecond, Units},
	traits::{
		clamp_zero_positive::ClampZeroPositive,
		handles_lights::{HandlesLights, Light, Responsive},
	},
};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct WallLight;

impl ExtraComponentsDefinition for WallLight {
	fn target_names() -> Vec<String> {
		vec!["LightData".to_owned()]
	}

	fn insert_bundle<TLights>(entity: &mut EntityCommands)
	where
		TLights: HandlesLights,
	{
		entity.try_insert((
			WallLight,
			TLights::responsive_light_bundle::<WallLight>(Responsive {
				range: Units::new(3.5),
				max: Intensity::new(8000.),
				change: IntensityChangePerSecond::new(4000.),
				light: Light::Point(|| PointLight {
					shadows_enabled: false,
					..default()
				}),
				light_on_material: || StandardMaterial {
					base_color: Color::WHITE,
					emissive: Color::linear_rgb(140.0, 140.0, 140.0).into(),
					..default()
				},
				light_off_material: || StandardMaterial {
					base_color: Color::BLACK,
					..default()
				},
			}),
		));
	}
}
