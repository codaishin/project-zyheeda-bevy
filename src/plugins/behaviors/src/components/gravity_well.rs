use super::LifeTime;
use bevy::{
	ecs::{component::Component, system::EntityCommands},
	math::primitives::Sphere,
	pbr::{AlphaMode, PbrBundle, StandardMaterial},
	render::{color::Color, mesh::Mesh},
	utils::default,
};
use bevy_rapier3d::geometry::Collider;
use common::{
	errors::Error,
	tools::UnitsPerSecond,
	traits::clamp_zero_positive::ClampZeroPositive,
};
use gravity::traits::{GetGravityEffectCollider, GetGravityPull};
use prefabs::traits::{AssetHandles, Instantiate};
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
		mut assets: impl AssetHandles,
	) -> Result<(), Error> {
		let base_color = Color::MIDNIGHT_BLUE;
		let emissive = base_color * 100.;

		on.insert((
			LifeTime(Duration::from_secs(5)),
			PbrBundle {
				mesh: assets
					.handle::<GravityWell>(&|| Mesh::from(Sphere::new(GravityWell::RADIUS))),
				material: assets.handle::<GravityWell>(&|| StandardMaterial {
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