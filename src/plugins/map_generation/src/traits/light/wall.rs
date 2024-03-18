use crate::{
	components::{Light, Wall},
	traits::Definition,
	types::ForChildren,
};
use bevy::{
	asset::Handle,
	ecs::system::EntityCommands,
	pbr::{PointLight, PointLightBundle, StandardMaterial},
	prelude::default,
	render::{color::Color, mesh::Mesh},
};
use common::errors::Error;
use prefabs::traits::{AssetKey, Instantiate, LightType};

impl Definition<Light<Wall>> for Light<Wall> {
	fn target_names() -> Vec<String> {
		vec![
			"LightNZ".to_owned(),
			"LightNX".to_owned(),
			"LightPZ".to_owned(),
			"LightPX".to_owned(),
		]
	}

	fn bundle() -> (Light<Wall>, ForChildren) {
		(Light::<Wall>::default(), ForChildren::from(true))
	}
}

impl Instantiate for Light<Wall> {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		_: impl FnMut(AssetKey, Mesh) -> Handle<Mesh>,
		mut get_material_handle: impl FnMut(AssetKey, StandardMaterial) -> Handle<StandardMaterial>,
	) -> Result<(), Error> {
		on.try_insert((
			PointLightBundle {
				point_light: PointLight {
					shadows_enabled: true,
					intensity: 4000.,
					..default()
				},
				..default()
			},
			get_material_handle(
				AssetKey::Light(LightType::Wall),
				StandardMaterial {
					base_color: Color::WHITE,
					emissive: Color::rgb_linear(23000.0, 23000.0, 23000.0),
					..default()
				},
			),
		));

		Ok(())
	}
}
