use super::LifeTime;
use bevy::{
	color::LinearRgba,
	ecs::{component::Component, system::EntityCommands},
	math::primitives::Sphere,
	pbr::{PbrBundle, StandardMaterial},
	render::{alpha::AlphaMode, mesh::Mesh},
	utils::default,
};
use bevy_rapier3d::geometry::Collider;
use common::{
	errors::Error,
	tools::UnitsPerSecond,
	traits::{cache::GetOrCreateTypeAsset, clamp_zero_positive::ClampZeroPositive},
};
use gravity::traits::{GetGravityEffectCollider, GetGravityPull};
use prefabs::traits::{GetOrCreateAssets, Instantiate};
use std::time::Duration;

#[derive(Component, Debug, PartialEq)]
pub struct GravityWell;

impl GravityWell {
	const RADIUS: f32 = 2.;
}

impl GetGravityPull for GravityWell {
	fn gravity_pull(&self) -> UnitsPerSecond {
		UnitsPerSecond::new(2.)
	}
}
impl GetGravityEffectCollider for GravityWell {
	fn gravity_effect_collider(&self) -> bevy_rapier3d::prelude::Collider {
		Collider::ball(GravityWell::RADIUS)
	}
}

impl Instantiate for GravityWell {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		mut assets: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		let base_color = LinearRgba::new(0.1, 0.1, 0.44, 1.);
		let emissive = base_color * 100.;

		on.insert((
			LifeTime(Duration::from_secs(5)),
			PbrBundle {
				mesh: assets.get_or_create_for::<GravityWell>(|| {
					Mesh::from(Sphere::new(GravityWell::RADIUS))
				}),
				material: assets.get_or_create_for::<GravityWell>(|| StandardMaterial {
					base_color: base_color.into(),
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
