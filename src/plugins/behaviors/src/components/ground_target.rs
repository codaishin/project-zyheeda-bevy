use bevy::{
	ecs::component::Component,
	math::{primitives::Plane3d, Ray3d, Vec3},
	transform::{components::Transform, TransformBundle},
};
use common::tools::Units;

#[derive(Component, Debug, PartialEq)]
pub struct GroundTarget {
	pub caster: Transform,
	pub target_ray: Ray3d,
	pub max_range: Units,
}

impl GroundTarget {
	fn intersect_ground_plane(self: &GroundTarget) -> Option<f32> {
		self.target_ray
			.intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::Y))
	}

	fn correct_for_max_range(self: &GroundTarget, target_translation: &mut Vec3) {
		let caster_translation = self.caster.translation;
		let target_direction = *target_translation - caster_translation;
		let max_range = *self.max_range;

		if target_direction.length() <= max_range {
			return;
		}

		*target_translation = caster_translation + target_direction.normalize() * max_range;
	}
}

impl From<GroundTarget> for TransformBundle {
	fn from(ground_target: GroundTarget) -> Self {
		let mut target_translation = match ground_target.intersect_ground_plane() {
			Some(toi) => ground_target.target_ray.origin + ground_target.target_ray.direction * toi,
			None => ground_target.caster.translation,
		};

		ground_target.correct_for_max_range(&mut target_translation);

		let transform = Transform::from_translation(target_translation)
			.with_rotation(ground_target.caster.rotation);

		TransformBundle::from(transform)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::math::{Ray3d, Vec3};
	use common::{tools::Units, traits::clamp_zero_positive::ClampZeroPositive};

	macro_rules! transform_bundle_equal {
		($transform:expr, $ground_target:expr) => {
			let expected = TransformBundle::from($transform);
			let got = TransformBundle::from($ground_target);

			assert_eq!((expected.local, expected.global), (got.local, got.global))
		};
	}

	#[test]
	fn set_transform_at_ray_intersecting_zero_elevation_plane() {
		transform_bundle_equal!(
			Transform::from_xyz(3., 0., 0.),
			GroundTarget {
				caster: Transform::from_xyz(10., 11., 12.),
				target_ray: Ray3d::new(Vec3::new(0., 5., 0.), Vec3::new(3., -5., 0.)),
				max_range: Units::new(42.),
			}
		);
	}

	#[test]
	fn set_transform_to_caster_transform_when_ray_not_hitting_zero_elevation_plane() {
		transform_bundle_equal!(
			Transform::from_xyz(10., 0., 12.),
			GroundTarget {
				caster: Transform::from_xyz(10., 0., 12.),
				target_ray: Ray3d::new(Vec3::new(0., 5., 0.), Vec3::Y),
				max_range: Units::new(42.),
			}
		);
	}

	#[test]
	fn limit_translation_to_be_within_max_range_from_caster() {
		transform_bundle_equal!(
			Transform::from_xyz(3., 0., 0.),
			GroundTarget {
				caster: Transform::from_xyz(2., 0., 0.),
				target_ray: Ray3d::new(Vec3::new(10., 3., 0.), Vec3::NEG_Y),
				max_range: Units::new(1.),
			}
		);
	}

	#[test]
	fn look_towards_caster_forward() {
		transform_bundle_equal!(
			Transform::from_xyz(10., 0., 0.).looking_to(Vec3::ONE, Vec3::Y),
			GroundTarget {
				caster: Transform::default().looking_to(Vec3::ONE, Vec3::Y),
				target_ray: Ray3d::new(Vec3::new(10., 3., 0.), Vec3::NEG_Y),
				max_range: Units::new(42.),
			}
		);
	}
}
