pub mod rapier_context;

use bevy::ecs::entity::Entity;
use bevy_rapier3d::{math::Real, prelude::RayIntersection};

#[derive(Debug, Default, PartialEq, PartialOrd, Clone, Copy)]
pub struct TimeOfImpact(pub Real);

pub trait CastRay<TRayData> {
	fn cast_ray(&self, ray_data: &TRayData) -> Option<(Entity, TimeOfImpact)>;
}

pub trait CastRayContinuously<TRayData> {
	fn cast_ray_continuously<F: FnMut(Entity, RayIntersection) -> bool>(
		&self,
		ray: &TRayData,
		callback: F,
	);
}

pub type SortedByTimeOfImpactAscending = Vec<(Entity, TimeOfImpact)>;

pub trait CastRayContinuouslySorted<TRayData> {
	fn cast_ray_continuously_sorted(&self, ray: &TRayData) -> SortedByTimeOfImpactAscending;
}

impl<T: CastRayContinuously<TRayData>, TRayData> CastRayContinuouslySorted<TRayData> for T {
	fn cast_ray_continuously_sorted(&self, ray: &TRayData) -> SortedByTimeOfImpactAscending {
		let mut results = Vec::new();

		self.cast_ray_continuously(ray, |entity, RayIntersection { time_of_impact, .. }| {
			results.push((entity, TimeOfImpact(time_of_impact)));
			true
		});

		// Safety: removing all the NaN values should make the unwrap below never fail.
		results.retain(|(_, toi)| !toi.0.is_nan());
		results.sort_by(|(_, toi_a), (_, toi_b)| toi_a.partial_cmp(toi_b).unwrap());

		results
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::math::Vec3;
	use bevy_rapier3d::parry::shape::FeatureId;
	use core::f32;

	struct _Ray;

	// Just for being descriptive in tests
	macro_rules! assert_no_error {
		($expr:expr) => {
			$expr
		};
	}

	fn intersection_toi(toi: f32) -> RayIntersection {
		RayIntersection {
			time_of_impact: toi,
			point: Vec3::default(),
			normal: Vec3::default(),
			feature: FeatureId::default(),
		}
	}

	#[test]
	fn sort_hits_ascending() {
		struct _CastRay;

		impl CastRayContinuously<_Ray> for _CastRay {
			fn cast_ray_continuously<F: FnMut(Entity, RayIntersection) -> bool>(
				&self,
				_: &_Ray,
				mut callback: F,
			) {
				callback(Entity::from_raw(1), intersection_toi(f32::INFINITY));
				callback(Entity::from_raw(2), intersection_toi(20.));
				callback(Entity::from_raw(3), intersection_toi(3.));
			}
		}

		let mock = _CastRay;

		let hits = mock.cast_ray_continuously_sorted(&_Ray);

		assert_eq!(
			vec![
				(Entity::from_raw(3), TimeOfImpact(3.)),
				(Entity::from_raw(2), TimeOfImpact(20.)),
				(Entity::from_raw(1), TimeOfImpact(f32::INFINITY))
			],
			hits
		)
	}

	#[test]
	fn remove_nan_toi_results_to_allow_sorting() {
		struct _CastRay;

		impl CastRayContinuously<_Ray> for _CastRay {
			fn cast_ray_continuously<F: FnMut(Entity, RayIntersection) -> bool>(
				&self,
				_: &_Ray,
				mut callback: F,
			) {
				callback(Entity::from_raw(666), intersection_toi(f32::NAN));
				callback(Entity::from_raw(2), intersection_toi(20.));
				callback(Entity::from_raw(3), intersection_toi(3.));
			}
		}

		let mock = _CastRay;

		let hits = assert_no_error!(mock.cast_ray_continuously_sorted(&_Ray));

		assert_eq!(
			vec![
				(Entity::from_raw(3), TimeOfImpact(3.)),
				(Entity::from_raw(2), TimeOfImpact(20.)),
			],
			hits
		);
	}
}
