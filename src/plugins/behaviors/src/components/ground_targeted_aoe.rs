use std::{f32::consts::PI, marker::PhantomData};

use super::{Contact, Projection};
use bevy::{
	color::Color,
	ecs::{component::Component, system::EntityCommands},
	math::{primitives::Sphere, Dir3, Quat, Ray3d, Vec3},
	pbr::{PbrBundle, StandardMaterial},
	prelude::{
		default,
		Annulus,
		BuildChildren,
		Bundle,
		Extrusion,
		InfinitePlane3d,
		TransformBundle,
	},
	render::{alpha::AlphaMode, mesh::Mesh},
	transform::components::Transform,
};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, ComputedColliderShape, Sensor};
use common::{
	components::ColliderRoot,
	errors::{Error, Level},
	tools::Units,
	traits::cache::GetOrCreateTypeAsset,
};
use prefabs::traits::{GetOrCreateAssets, Instantiate};

#[derive(Component, Debug, PartialEq, Clone)]
pub struct GroundTargetedAoe<T> {
	pub phantom_data: PhantomData<T>,
	pub caster: Transform,
	pub target_ray: Ray3d,
	pub max_range: Units,
	pub radius: Units,
}

impl GroundTargetedAoe<()> {
	pub const DEFAULT_TARGET_RAY: Ray3d = Ray3d {
		origin: Vec3::Y,
		direction: Dir3::NEG_Y,
	};
}

pub struct Args {
	pub caster: Transform,
	pub target_ray: Ray3d,
	pub max_range: Units,
	pub radius: Units,
}

impl<T> GroundTargetedAoe<T> {
	fn intersect_ground_plane(self: &GroundTargetedAoe<T>) -> Option<f32> {
		self.target_ray
			.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
	}

	fn correct_for_max_range(self: &GroundTargetedAoe<T>, target_translation: &mut Vec3) {
		let caster_translation = self.caster.translation;
		let target_direction = *target_translation - caster_translation;
		let max_range = *self.max_range;

		if target_direction.length() <= max_range {
			return;
		}

		*target_translation = caster_translation + target_direction.normalize() * max_range;
	}
}

trait ColliderComponents {
	fn collider_components(&self) -> Result<impl Bundle, Error>;
}

impl<T: Default> Default for GroundTargetedAoe<T> {
	fn default() -> Self {
		Self {
			phantom_data: default(),
			caster: default(),
			target_ray: GroundTargetedAoe::DEFAULT_TARGET_RAY,
			max_range: default(),
			radius: default(),
		}
	}
}

impl Instantiate for GroundTargetedAoe<Contact> {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		mut assets: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		let base_color = Color::srgb(0.1, 0.1, 0.44);
		let emissive = base_color.to_linear() * 100.;
		let collider = self.collider_components()?;

		on.try_insert(PbrBundle {
			mesh: assets.get_or_create_for::<GroundTargetedAoe<Contact>>(|| {
				Mesh::from(Sphere::new(*self.radius))
			}),
			material: assets.get_or_create_for::<GroundTargetedAoe<Contact>>(|| StandardMaterial {
				base_color,
				emissive,
				alpha_mode: AlphaMode::Add,
				..default()
			}),
			transform: Transform::from(self),
			..default()
		})
		.with_children(|parent| {
			parent.spawn((ColliderRoot(parent.parent_entity()), collider));
		});

		Ok(())
	}
}

impl Instantiate for GroundTargetedAoe<Projection> {
	fn instantiate(&self, on: &mut EntityCommands, _: impl GetOrCreateAssets) -> Result<(), Error> {
		let collider = self.collider_components()?;

		on.try_insert(collider);

		Ok(())
	}
}

impl<T> From<&GroundTargetedAoe<T>> for Transform {
	fn from(ground_target: &GroundTargetedAoe<T>) -> Self {
		let mut target_translation = match ground_target.intersect_ground_plane() {
			Some(toi) => ground_target.target_ray.origin + ground_target.target_ray.direction * toi,
			None => ground_target.caster.translation,
		};

		ground_target.correct_for_max_range(&mut target_translation);

		Transform::from_translation(target_translation).with_rotation(ground_target.caster.rotation)
	}
}

impl ColliderComponents for GroundTargetedAoe<Projection> {
	fn collider_components(&self) -> Result<impl Bundle, Error> {
		Ok((
			TransformBundle::default(),
			Collider::ball(*self.radius),
			ActiveEvents::COLLISION_EVENTS,
			Sensor,
		))
	}
}

impl ColliderComponents for GroundTargetedAoe<Contact> {
	fn collider_components(&self) -> Result<impl Bundle, Error> {
		let transform = Transform::default().with_rotation(Quat::from_axis_angle(Vec3::X, PI / 2.));
		let ring = Annulus::new(*self.radius - 0.1, *self.radius + 0.1);
		let torus = Mesh::from(Extrusion::new(ring, 2.));
		let collider = Collider::from_bevy_mesh(&torus, &ComputedColliderShape::TriMesh);

		let Some(collider) = collider else {
			return Err(Error {
				msg: "Cannot create ground targeted AoE contact collider".to_owned(),
				lvl: Level::Error,
			});
		};

		Ok((
			TransformBundle::from(transform),
			collider,
			ActiveEvents::COLLISION_EVENTS,
			Sensor,
		))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::math::{Ray3d, Vec3};
	use common::{tools::Units, traits::clamp_zero_positive::ClampZeroPositive};

	#[derive(Default)]
	struct _T;

	#[test]
	fn set_transform_at_ray_intersecting_zero_elevation_plane() {
		assert_eq!(
			Transform::from_xyz(3., 0., 0.),
			Transform::from(&GroundTargetedAoe::<_T> {
				caster: Transform::from_xyz(10., 11., 12.),
				target_ray: Ray3d::new(Vec3::new(0., 5., 0.), Vec3::new(3., -5., 0.)),
				max_range: Units::new(42.),
				..default()
			})
		);
	}

	#[test]
	fn set_transform_to_caster_transform_when_ray_not_hitting_zero_elevation_plane() {
		assert_eq!(
			Transform::from_xyz(10., 0., 12.),
			Transform::from(&GroundTargetedAoe::<_T> {
				caster: Transform::from_xyz(10., 0., 12.),
				target_ray: Ray3d::new(Vec3::new(0., 5., 0.), Vec3::Y),
				max_range: Units::new(42.),
				..default()
			})
		);
	}

	#[test]
	fn limit_translation_to_be_within_max_range_from_caster() {
		assert_eq!(
			Transform::from_xyz(3., 0., 0.),
			Transform::from(&GroundTargetedAoe::<_T> {
				caster: Transform::from_xyz(2., 0., 0.),
				target_ray: Ray3d::new(Vec3::new(10., 3., 0.), Vec3::NEG_Y),
				max_range: Units::new(1.),
				..default()
			})
		);
	}

	#[test]
	fn look_towards_caster_forward() {
		assert_eq!(
			Transform::from_xyz(10., 0., 0.).looking_to(Vec3::ONE, Vec3::Y),
			Transform::from(&GroundTargetedAoe::<_T> {
				caster: Transform::default().looking_to(Vec3::ONE, Vec3::Y),
				target_ray: Ray3d::new(Vec3::new(10., 3., 0.), Vec3::NEG_Y),
				max_range: Units::new(42.),
				..default()
			})
		);
	}
}
