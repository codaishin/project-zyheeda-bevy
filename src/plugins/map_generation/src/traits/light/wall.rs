use crate::{
	components::{Light, Wall},
	traits::ExtraComponentsDefinition,
};
use bevy::{
	color::Color,
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	pbr::{PointLight, PointLightBundle, StandardMaterial},
	prelude::default,
	render::view::Visibility,
};
use common::{
	errors::Error,
	tools::{Intensity, IntensityChangePerSecond, Units},
	traits::{
		cache::GetOrCreateTypeAsset,
		clamp_zero_positive::ClampZeroPositive,
		prefab::{GetOrCreateAssets, Instantiate},
		try_insert_on::TryInsertOn,
	},
};
use light::components::{ResponsiveLight, ResponsiveLightData};

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

impl Instantiate for Light<Wall> {
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		mut assets: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		let model = entity.id();
		let mut commands = entity.commands();

		let light_on_material = assets.get_or_create_for::<WallLightOn>(|| StandardMaterial {
			base_color: Color::WHITE,
			emissive: Color::linear_rgb(140.0, 140.0, 140.0).into(),
			..default()
		});
		let light_off_material = assets.get_or_create_for::<WallLightOff>(|| StandardMaterial {
			base_color: Color::BLACK,
			..default()
		});

		let light = commands
			.spawn(PointLightBundle {
				point_light: PointLight {
					shadows_enabled: false,
					intensity: 0.,
					..default()
				},
				visibility: Visibility::Hidden,
				..default()
			})
			.set_parent(model)
			.id();
		commands.try_insert_on(
			model,
			(
				light_off_material.clone(),
				ResponsiveLight {
					model,
					light,
					data: ResponsiveLightData {
						range: Units::new(3.5),
						light_on_material,
						light_off_material,
						max: Intensity::new(8000.),
						change: IntensityChangePerSecond::new(4000.),
					},
				},
			),
		);

		Ok(())
	}
}
