pub mod read_rapier_context;
pub mod system_input;

use bevy::{ecs::entity::Entity, math::Vec3};
use bevy_rapier3d::prelude::RayIntersection;
use common::{
	errors::{ErrorData, Level},
	traits::handles_physics::{IsNaN, TimeOfImpact},
};
use zyheeda_core::prelude::Sorted;

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct RayHit {
	pub(crate) entity: Entity,
	pub(crate) toi: TimeOfImpact,
}

impl PartialOrd for RayHit {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for RayHit {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.toi.cmp(&other.toi)
	}
}

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
	fn cast_ray_continuously_sorted(
		&self,
		ray: &TRayData,
	) -> Result<Sorted<RayHit>, InvalidIntersections>;
}

impl<T, TRayData> CastRayContinuouslySorted<TRayData> for T
where
	T: CastRayContinuously<TRayData>,
{
	fn cast_ray_continuously_sorted(
		&self,
		ray: &TRayData,
	) -> Result<Sorted<RayHit>, InvalidIntersections> {
		let mut results = Sorted::default();
		let mut invalid_intersections = vec![];

		self.cast_ray_continuously(ray, |entity, intersection| {
			match TimeOfImpact::try_from(intersection.time_of_impact) {
				Ok(toi) => results.push(RayHit { entity, toi }),
				Err(IsNaN) => invalid_intersections.push(intersection.point),
			};

			true
		});

		if !invalid_intersections.is_empty() {
			return Err(InvalidIntersections(invalid_intersections));
		}

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
	use testing::{assert_no_panic, fake_entity};

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
	fn sort_hits_ascending() -> Result<(), InvalidIntersections> {
		struct _CastRay;

		impl CastRayContinuously<_Ray> for _CastRay {
			fn cast_ray_continuously<F: FnMut(Entity, RayIntersection) -> bool>(
				&self,
				_: &_Ray,
				mut callback: F,
			) {
				callback(fake_entity!(1), intersection_toi(f32::INFINITY));
				callback(fake_entity!(2), intersection_toi(20.));
				callback(fake_entity!(3), intersection_toi(3.));
			}
		}

		let mock = _CastRay;

		let hits = mock.cast_ray_continuously_sorted(&_Ray)?;

		assert_eq!(
			&[
				RayHit {
					entity: fake_entity!(3),
					toi: toi!(3.)
				},
				RayHit {
					entity: fake_entity!(2),
					toi: toi!(20.)
				},
				RayHit {
					entity: fake_entity!(1),
					toi: toi!(f32::INFINITY)
				},
			],
			&*hits
		);
		Ok(())
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

				callback(fake_entity!(666), invalid);
				callback(fake_entity!(2), intersection_toi(20.));
				callback(fake_entity!(3), intersection_toi(3.));
			}
		}

		let mock = _CastRay;

		let hits = assert_no_panic!(mock.cast_ray_continuously_sorted(&_Ray));

		assert_eq!(Err(InvalidIntersections(vec![Vec3::new(1., 2., 3.)])), hits);
	}
}
