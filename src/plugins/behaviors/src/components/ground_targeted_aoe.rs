use bevy::{
	ecs::{component::Component, system::EntityCommands},
	math::{Dir3, Quat, Ray3d, Vec3},
	prelude::{
		default,
		Annulus,
		BuildChildren,
		Bundle,
		Extrusion,
		InfinitePlane3d,
		SpatialBundle,
		TransformBundle,
	},
	render::mesh::Mesh,
	transform::components::Transform,
};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, ComputedColliderShape, Sensor};
use common::{
	components::{AssetModel, ColliderRoot},
	errors::{Error, Level},
	tools::Units,
};
use prefabs::traits::{GetOrCreateAssets, Instantiate};
use std::f32::consts::PI;

#[derive(Component, Debug, PartialEq, Clone)]
pub struct GroundTargetedAoeContact {
	pub caster: Transform,
	pub target_ray: Ray3d,
	pub max_range: Units,
	pub radius: Units,
}

impl GroundTargetedAoeContact {
	pub const DEFAULT_TARGET_RAY: Ray3d = Ray3d {
		origin: Vec3::Y,
		direction: Dir3::NEG_Y,
	};
}

impl GroundTargetedAoeContact {
	fn intersect_ground_plane(&self) -> Option<f32> {
		self.target_ray
			.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
	}

	fn correct_for_max_range(&self, target_translation: &mut Vec3) {
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

impl Default for GroundTargetedAoeContact {
	fn default() -> Self {
		Self {
			caster: default(),
			target_ray: GroundTargetedAoeContact::DEFAULT_TARGET_RAY,
			max_range: default(),
			radius: default(),
		}
	}
}

impl Instantiate for GroundTargetedAoeContact {
	fn instantiate(&self, on: &mut EntityCommands, _: impl GetOrCreateAssets) -> Result<(), Error> {
		let collider = self.collider_components()?;
		let transform = Transform::from(self).with_scale(Vec3::splat(*self.radius * 2.));

		on.try_insert((
			AssetModel("models/sphere.glb#Scene0"),
			SpatialBundle::from(transform),
		))
		.with_children(|parent| {
			parent.spawn((ColliderRoot(parent.parent_entity()), collider));
		});

		Ok(())
	}
}

impl From<&GroundTargetedAoeContact> for Transform {
	fn from(ground_target: &GroundTargetedAoeContact) -> Self {
		let mut target_translation = match ground_target.intersect_ground_plane() {
			Some(toi) => ground_target.target_ray.origin + ground_target.target_ray.direction * toi,
			None => ground_target.caster.translation,
		};

		ground_target.correct_for_max_range(&mut target_translation);

		Transform::from_translation(target_translation).with_rotation(ground_target.caster.rotation)
	}
}

impl ColliderComponents for GroundTargetedAoeContact {
	fn collider_components(&self) -> Result<impl Bundle, Error> {
		let transform = Transform::default().with_rotation(Quat::from_axis_angle(Vec3::X, PI / 2.));
		let ring = Annulus::new(0.49, 0.51);
		let torus = Mesh::from(Extrusion::new(ring, 0.5));
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

#[derive(Component, Debug, PartialEq, Clone)]
pub struct GroundTargetedAoeProjection {
	pub radius: Units,
}

impl Instantiate for GroundTargetedAoeProjection {
	fn instantiate(&self, on: &mut EntityCommands, _: impl GetOrCreateAssets) -> Result<(), Error> {
		let collider = self.collider_components()?;

		on.try_insert(collider);

		Ok(())
	}
}

impl ColliderComponents for GroundTargetedAoeProjection {
	fn collider_components(&self) -> Result<impl Bundle, Error> {
		Ok((
			TransformBundle::default(),
			Collider::ball(0.5),
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

	#[test]
	fn set_transform_at_ray_intersecting_zero_elevation_plane() {
		assert_eq!(
			Transform::from_xyz(3., 0., 0.),
			Transform::from(&GroundTargetedAoeContact {
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
			Transform::from(&GroundTargetedAoeContact {
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
			Transform::from(&GroundTargetedAoeContact {
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
			Transform::from(&GroundTargetedAoeContact {
				caster: Transform::default().looking_to(Vec3::ONE, Vec3::Y),
				target_ray: Ray3d::new(Vec3::new(10., 3., 0.), Vec3::NEG_Y),
				max_range: Units::new(42.),
				..default()
			})
		);
	}
}
