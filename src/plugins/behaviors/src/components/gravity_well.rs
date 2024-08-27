use super::LifeTime;
use bevy::{
	color::Color,
	ecs::{component::Component, system::EntityCommands},
	math::primitives::Sphere,
	pbr::{PbrBundle, StandardMaterial},
	render::{alpha::AlphaMode, mesh::Mesh},
	utils::default,
};
use common::{errors::Error, traits::cache::GetOrCreateTypeAsset};
use prefabs::traits::{GetOrCreateAssets, Instantiate};
use std::time::Duration;

#[derive(Component, Debug, PartialEq)]
pub struct GravityWell;

impl GravityWell {
	const RADIUS: f32 = 2.;
}

impl Instantiate for GravityWell {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		mut assets: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		let base_color = Color::srgb(0.1, 0.1, 0.44);
		let emissive = base_color.to_linear() * 100.;

		on.insert((
			LifeTime(Duration::from_secs(5)),
			PbrBundle {
				mesh: assets.get_or_create_for::<GravityWell>(|| {
					Mesh::from(Sphere::new(GravityWell::RADIUS))
				}),
				material: assets.get_or_create_for::<GravityWell>(|| StandardMaterial {
					base_color,
					emissive,
					alpha_mode: AlphaMode::Add,
					..default()
				}),
				..default()
			},
		));

		Ok(())
	}
}
