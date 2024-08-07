use super::ProjectileBehavior;
use crate::components::{MovementConfig, MovementMode, Plasma, Projectile};
use bevy::{
	self,
	color::Color,
	hierarchy::BuildChildren,
	math::{Dir3, Vec3},
	pbr::{PbrBundle, PointLight, PointLightBundle, StandardMaterial},
	transform::components::Transform,
	utils::default,
};
use bevy_rapier3d::{
	dynamics::RigidBody,
	geometry::{Collider, Sensor},
};
use common::{
	bundles::ColliderTransformBundle,
	components::ColliderRoot,
	errors::Error,
	tools::UnitsPerSecond,
	traits::{cache::GetOrCreateTypeAsset, clamp_zero_positive::ClampZeroPositive},
};
use interactions::components::{DealsDamage, Fragile};
use prefabs::traits::{sphere, GetOrCreateAssets, Instantiate};

impl<T> ProjectileBehavior for Projectile<T> {
	fn direction(&self) -> Dir3 {
		self.direction
	}

	fn range(&self) -> f32 {
		self.range
	}
}

const PLASMA_RADIUS: f32 = 0.05;

impl Instantiate for Projectile<Plasma> {
	fn instantiate(
		&self,
		on: &mut bevy::ecs::system::EntityCommands,
		mut assets: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		let transform = Transform::from_translation(Vec3::ZERO);
		let color = Color::srgb(0., 1., 1.);
		let mesh = assets.get_or_create_for::<Projectile<Plasma>>(|| sphere(PLASMA_RADIUS));
		let material = assets.get_or_create_for::<Projectile<Plasma>>(|| StandardMaterial {
			emissive: color.to_linear() * 2300.0,
			..default()
		});

		on.try_insert((
			RigidBody::Fixed,
			DealsDamage(1),
			Fragile,
			MovementConfig::Constant {
				mode: MovementMode::Fast,
				speed: UnitsPerSecond::new(15.),
			},
		))
		.with_children(|parent| {
			parent.spawn(PbrBundle {
				transform,
				mesh,
				material,
				..default()
			});
			parent.spawn((
				ColliderTransformBundle::new_static_collider(
					transform,
					Collider::ball(PLASMA_RADIUS),
				),
				Sensor,
				ColliderRoot(parent.parent_entity()),
			));
			parent.spawn(PointLightBundle {
				point_light: PointLight {
					color,
					intensity: 8000.,
					shadows_enabled: true,
					..default()
				},
				..default()
			});
		});

		Ok(())
	}
}
