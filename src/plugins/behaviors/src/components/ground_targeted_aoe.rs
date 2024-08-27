use bevy::{
	ecs::{component::Component, system::EntityCommands},
	math::{Dir3, Ray3d, Vec3},
	prelude::{default, InfinitePlane3d, TransformBundle},
	transform::components::Transform,
};
use bevy_rapier3d::prelude::Collider;
use common::{errors::Error, tools::Units};
use prefabs::traits::{GetOrCreateAssets, Instantiate};

#[derive(Component, Debug, PartialEq, Clone)]
pub struct GroundTargetedAoe {
	pub caster: Transform,
	pub target_ray: Ray3d,
	pub max_range: Units,
	pub radius: Units,
}

impl GroundTargetedAoe {
	pub const DEFAULT_TARGET_RAY: Ray3d = Ray3d {
		origin: Vec3::Y,
		direction: Dir3::NEG_Y,
	};

	fn intersect_ground_plane(self: &GroundTargetedAoe) -> Option<f32> {
		self.target_ray
			.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
	}

	fn correct_for_max_range(self: &GroundTargetedAoe, target_translation: &mut Vec3) {
		let caster_translation = self.caster.translation;
		let target_direction = *target_translation - caster_translation;
		let max_range = *self.max_range;

		if target_direction.length() <= max_range {
			return;
		}

		*target_translation = caster_translation + target_direction.normalize() * max_range;
	}
}

impl Default for GroundTargetedAoe {
	fn default() -> Self {
		Self {
			caster: default(),
			target_ray: Self::DEFAULT_TARGET_RAY,
			max_range: default(),
			radius: default(),
		}
	}
}

impl Instantiate for GroundTargetedAoe {
	fn instantiate(&self, on: &mut EntityCommands, _: impl GetOrCreateAssets) -> Result<(), Error> {
		on.try_insert((
			TransformBundle::from(self.clone()),
			Collider::from(self.clone()),
		));

		Ok(())
	}
}

impl From<GroundTargetedAoe> for TransformBundle {
	fn from(ground_target: GroundTargetedAoe) -> Self {
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

impl From<GroundTargetedAoe> for Collider {
	fn from(ground_target: GroundTargetedAoe) -> Self {
		Collider::ball(*ground_target.radius)
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
			GroundTargetedAoe {
				caster: Transform::from_xyz(10., 11., 12.),
				target_ray: Ray3d::new(Vec3::new(0., 5., 0.), Vec3::new(3., -5., 0.)),
				max_range: Units::new(42.),
				..default()
			}
		);
	}

	#[test]
	fn set_transform_to_caster_transform_when_ray_not_hitting_zero_elevation_plane() {
		transform_bundle_equal!(
			Transform::from_xyz(10., 0., 12.),
			GroundTargetedAoe {
				caster: Transform::from_xyz(10., 0., 12.),
				target_ray: Ray3d::new(Vec3::new(0., 5., 0.), Vec3::Y),
				max_range: Units::new(42.),
				..default()
			}
		);
	}

	#[test]
	fn limit_translation_to_be_within_max_range_from_caster() {
		transform_bundle_equal!(
			Transform::from_xyz(3., 0., 0.),
			GroundTargetedAoe {
				caster: Transform::from_xyz(2., 0., 0.),
				target_ray: Ray3d::new(Vec3::new(10., 3., 0.), Vec3::NEG_Y),
				max_range: Units::new(1.),
				..default()
			}
		);
	}

	#[test]
	fn look_towards_caster_forward() {
		transform_bundle_equal!(
			Transform::from_xyz(10., 0., 0.).looking_to(Vec3::ONE, Vec3::Y),
			GroundTargetedAoe {
				caster: Transform::default().looking_to(Vec3::ONE, Vec3::Y),
				target_ray: Ray3d::new(Vec3::new(10., 3., 0.), Vec3::NEG_Y),
				max_range: Units::new(42.),
				..default()
			}
		);
	}

	#[test]
	fn collider_from_ground_target_aoe() {
		let aoe = GroundTargetedAoe {
			radius: Units::new(33.),
			..default()
		};

		let collider = Collider::from(aoe);

		assert_eq!(Some(33.), collider.as_ball().map(|b| b.radius()));
	}
}
