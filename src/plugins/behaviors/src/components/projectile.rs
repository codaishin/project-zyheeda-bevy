use super::{MovementConfig, MovementMode};
use crate::traits::ProjectileBehavior;
use bevy::{
	self,
	color::Color,
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	math::{Dir3, Vec3},
	pbr::{PbrBundle, PointLight, PointLightBundle, StandardMaterial},
	prelude::{Component, Transform},
	utils::default,
};
use bevy_rapier3d::{
	geometry::{Collider, Sensor},
	prelude::RigidBody,
};
use common::{
	bundles::ColliderTransformBundle,
	components::ColliderRoot,
	errors::Error,
	test_tools::utils::ApproxEqual,
	tools::UnitsPerSecond,
	traits::{cache::GetOrCreateTypeAsset, clamp_zero_positive::ClampZeroPositive},
};
use interactions::components::{DealsDamage, Fragile};
use prefabs::traits::{sphere, GetOrCreateAssets, Instantiate};
use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub struct Plasma;

#[derive(Component, Debug, PartialEq)]
pub struct Projectile<T> {
	pub direction: Dir3,
	pub range: f32,
	phantom_data: PhantomData<T>,
}

impl<T> Default for Projectile<T> {
	fn default() -> Self {
		Self {
			direction: Dir3::NEG_Z,
			range: Default::default(),
			phantom_data: Default::default(),
		}
	}
}

impl<T> ApproxEqual<f32> for Projectile<T> {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		self.direction
			.as_vec3()
			.approx_equal(&other.direction.as_vec3(), tolerance)
			&& self.range == other.range
	}
}

impl<T> Projectile<T> {
	pub fn new(direction: Dir3, range: f32) -> Self {
		Self {
			direction,
			range,
			phantom_data: PhantomData,
		}
	}
}

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
		on: &mut EntityCommands,
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
