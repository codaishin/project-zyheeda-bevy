use super::{GetContinuousSortedRayCaster, GetRayCaster};
use bevy::prelude::*;

impl<TRayData, T> GetRayCaster<TRayData> for In<T>
where
	T: GetRayCaster<TRayData>,
{
	type TError = T::TError;
	type TRayCaster<'a>
		= T::TRayCaster<'a>
	where
		Self: 'a;

	fn get_ray_caster(&self) -> Result<Self::TRayCaster<'_>, Self::TError> {
		let In(value) = self;
		value.get_ray_caster()
	}
}

impl<TRayData, T> GetContinuousSortedRayCaster<TRayData> for In<T>
where
	T: GetContinuousSortedRayCaster<TRayData>,
{
	type TError = T::TError;
	type TRayCaster<'a>
		= T::TRayCaster<'a>
	where
		Self: 'a;

	fn get_continuous_sorted_ray_caster(&self) -> Result<Self::TRayCaster<'_>, Self::TError> {
		let In(value) = self;
		value.get_continuous_sorted_ray_caster()
	}
}
