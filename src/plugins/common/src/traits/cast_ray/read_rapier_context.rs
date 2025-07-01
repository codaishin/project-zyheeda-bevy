use super::{
	CastRay,
	CastRayContinuously,
	GetContinuousSortedRayCaster,
	GetRayCaster,
	TimeOfImpact,
};
use bevy::{
	ecs::{entity::Entity, error::BevyError},
	math::Ray3d,
};
use bevy_rapier3d::{
	math::Real,
	pipeline::QueryFilterFlags,
	plugin::{RapierContext, ReadRapierContext},
	prelude::QueryFilter,
};

impl<T> GetRayCaster<T> for ReadRapierContext<'_, '_>
where
	for<'a> RapierContext<'a>: CastRay<T>,
{
	type TError = BevyError;
	type TRayCaster<'a>
		= RapierContext<'a>
	where
		Self: 'a;

	fn get_ray_caster(&self) -> Result<Self::TRayCaster<'_>, Self::TError> {
		self.single()
	}
}

impl<T> GetContinuousSortedRayCaster<T> for ReadRapierContext<'_, '_>
where
	for<'a> RapierContext<'a>: CastRayContinuously<T>,
{
	type TError = BevyError;
	type TRayCaster<'a>
		= RapierContext<'a>
	where
		Self: 'a;

	fn get_continuous_sorted_ray_caster(&self) -> Result<Self::TRayCaster<'_>, Self::TError> {
		self.single()
	}
}

impl CastRay<(Ray3d, NoSensors)> for RapierContext<'_> {
	fn cast_ray(&self, (ray, _): &(Ray3d, NoSensors)) -> Option<(Entity, TimeOfImpact)> {
		self.cast_ray(
			ray.origin,
			ray.direction.into(),
			Real::MAX,
			true,
			QueryFilterFlags::EXCLUDE_SENSORS.into(),
		)
		.map(|(entity, toi)| (entity, TimeOfImpact(toi)))
	}
}

impl CastRay<(Ray3d, NoSensors, ExcludeRigidBody)> for RapierContext<'_> {
	fn cast_ray(
		&self,
		(ray, _, ExcludeRigidBody(rigid_body)): &(Ray3d, NoSensors, ExcludeRigidBody),
	) -> Option<(Entity, TimeOfImpact)> {
		let query_filter =
			QueryFilter::from(QueryFilterFlags::EXCLUDE_SENSORS).exclude_rigid_body(*rigid_body);

		self.cast_ray(
			ray.origin,
			ray.direction.into(),
			Real::MAX,
			true,
			query_filter,
		)
		.map(|(entity, toi)| (entity, TimeOfImpact(toi)))
	}
}

impl CastRay<(Ray3d, OnlySensors)> for RapierContext<'_> {
	fn cast_ray(&self, (ray, _): &(Ray3d, OnlySensors)) -> Option<(Entity, TimeOfImpact)> {
		let query_filter = QueryFilter::from(QueryFilterFlags::EXCLUDE_SOLIDS);

		self.cast_ray(
			ray.origin,
			ray.direction.into(),
			Real::MAX,
			true,
			query_filter,
		)
		.map(|(entity, toi)| (entity, TimeOfImpact(toi)))
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ExcludeRigidBody(pub Entity);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct NoSensors;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct OnlySensors;
