use crate::traits::ExtraComponentsDefinition;
use bevy::prelude::*;
use common::{
	errors::Error,
	tools::{Intensity, IntensityChangePerSecond, Units},
	traits::{
		clamp_zero_positive::ClampZeroPositive,
		handles_lights::{HandlesLights, Responsive},
		prefab::Prefab,
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
		let model = entity.id();
		entity.try_insert(WallLight).with_children(|parent| {
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
			parent.spawn(TLights::responsive_light_bundle::<Self>(Responsive {
				model,
				light,
				range: Units::new(3.5),
				light_on_material: || StandardMaterial {
					base_color: Color::WHITE,
					emissive: Color::linear_rgb(140.0, 140.0, 140.0).into(),
					..default()
				},
				light_off_material: || StandardMaterial {
					base_color: Color::BLACK,
					..default()
				},
				max: Intensity::new(8000.),
				change: IntensityChangePerSecond::new(4000.),
			}));
		});
	}
}

impl<TLights> Prefab<TLights> for WallLight
where
	TLights: HandlesLights,
{
	fn instantiate_on(&self, entity: &mut EntityCommands) -> Result<(), Error> {
		let model = entity.id();

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
			parent.spawn(TLights::responsive_light_bundle::<Self>(Responsive {
				model,
				light,
				range: Units::new(3.5),
				light_on_material: || StandardMaterial {
					base_color: Color::WHITE,
					emissive: Color::linear_rgb(140.0, 140.0, 140.0).into(),
					..default()
				},
				light_off_material: || StandardMaterial {
					base_color: Color::BLACK,
					..default()
				},
				max: Intensity::new(8000.),
				change: IntensityChangePerSecond::new(4000.),
			}));
		});

		Ok(())
	}
}
