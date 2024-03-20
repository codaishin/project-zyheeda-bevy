use crate::{
	components::{Light, Wall},
	traits::Definition,
};
use bevy::{
	asset::Handle,
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	pbr::{PointLight, PointLightBundle, StandardMaterial},
	prelude::default,
	render::{color::Color, mesh::Mesh, view::Visibility},
};
use common::{errors::Error, traits::try_insert_on::TryInsertOn};
use light::components::ResponsiveLight;
use prefabs::traits::{AssetKey, Instantiate, LightStatus, LightType};

impl Definition for Light<Wall> {
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

impl Instantiate for Light<Wall> {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		_: impl FnMut(AssetKey, Mesh) -> Handle<Mesh>,
		mut get_material_handle: impl FnMut(AssetKey, StandardMaterial) -> Handle<StandardMaterial>,
	) -> Result<(), Error> {
		let model = on.id();
		let mut commands = on.commands();

		let light_on_material = get_material_handle(
			AssetKey::Light(LightType::Wall(LightStatus::On)),
			StandardMaterial {
				base_color: Color::WHITE,
				emissive: Color::rgb_linear(14000.0, 14000.0, 14000.0),
				..default()
			},
		);
		let light_off_material = get_material_handle(
			AssetKey::Light(LightType::Wall(LightStatus::Off)),
			StandardMaterial {
				base_color: Color::BLACK,
				..default()
			},
		);

		let light = commands
			.spawn(PointLightBundle {
				point_light: PointLight {
					shadows_enabled: false,
					intensity: 4000.,
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
					range: 3.5,
					model,
					light,
					light_on_material,
					light_off_material,
				},
			),
		);

		Ok(())
	}
}
