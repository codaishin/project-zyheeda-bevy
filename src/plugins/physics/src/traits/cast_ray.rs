pub mod read_rapier_context;
pub mod system_input;

use bevy::{ecs::entity::Entity, math::Vec3};
use bevy_rapier3d::prelude::RayIntersection;
use common::{
	errors::{ErrorData, Level},
	traits::handles_physics::{IsNaN, TimeOfImpact},
};

pub trait GetContinuousSortedRayCaster<TRayData> {
	type TError;
	type TRayCaster<'a>: CastRayContinuouslySorted<TRayData>
	where
		Self: 'a;

	fn get_continuous_sorted_ray_caster(&self) -> Result<Self::TRayCaster<'_>, Self::TError>;
}

pub trait CastRayContinuously<TRayData> {
	fn cast_ray_continuously<F: FnMut(Entity, RayIntersection) -> bool>(
		&self,
		ray: &TRayData,
		callback: F,
	);
}

pub type SortedByTimeOfImpactAscending = Result<Vec<(Entity, TimeOfImpact)>, InvalidIntersections>;

#[derive(Debug, PartialEq, Clone)]
pub struct InvalidIntersections(pub Vec<Vec3>);

impl ErrorData for InvalidIntersections {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl std::fmt::Display {
		"Raycast Intersections sorting failed"
	}

	fn into_details(self) -> impl std::fmt::Display {
		format!("Intersections with `NaN` time of impact: {:?}", self.0)
	}
}

pub trait CastRayContinuouslySorted<TRayData> {
	fn cast_ray_continuously_sorted(&self, ray: &TRayData) -> SortedByTimeOfImpactAscending;
}

impl<T, TRayData> CastRayContinuouslySorted<TRayData> for T
where
	T: CastRayContinuously<TRayData>,
{
	fn cast_ray_continuously_sorted(&self, ray: &TRayData) -> SortedByTimeOfImpactAscending {
		let mut results = vec![];
		let mut invalid_intersections = vec![];

		self.cast_ray_continuously(ray, |entity, intersection| {
			match TimeOfImpact::try_from(intersection.time_of_impact) {
				Ok(time_of_impact) => results.push((entity, time_of_impact)),
				Err(IsNaN) => invalid_intersections.push(intersection.point),
			};

			true
		});

		if !invalid_intersections.is_empty() {
			return Err(InvalidIntersections(invalid_intersections));
		}

		results.sort_by_key(|(.., toi)| *toi);

		Ok(results)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::math::Vec3;
	use bevy_rapier3d::parry::shape::FeatureId;
	use common::toi;
	use core::f32;
	use testing::assert_no_panic;

	struct _Ray;

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
			Ok(vec![
				(Entity::from_raw(3), toi!(3.)),
				(Entity::from_raw(2), toi!(20.)),
				(Entity::from_raw(1), toi!(f32::INFINITY))
			]),
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
				let mut invalid = intersection_toi(f32::NAN);
				invalid.point = Vec3::new(1., 2., 3.);

				callback(Entity::from_raw(666), invalid);
				callback(Entity::from_raw(2), intersection_toi(20.));
				callback(Entity::from_raw(3), intersection_toi(3.));
			}
		}

		let mock = _CastRay;

		let hits = assert_no_panic!(mock.cast_ray_continuously_sorted(&_Ray));

		assert_eq!(Err(InvalidIntersections(vec![Vec3::new(1., 2., 3.)])), hits);
	}
}
