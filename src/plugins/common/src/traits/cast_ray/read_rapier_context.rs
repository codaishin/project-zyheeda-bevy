use super::{
	CastRay,
	CastRayContinuously,
	GetContinuousSortedRayCaster,
	GetRayCaster,
	TimeOfImpact,
};
use crate::tools::exclude_rigid_body::ExcludeRigidBody;
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

impl CastRay<Ray3d> for RapierContext<'_> {
	fn cast_ray(&self, ray: &Ray3d) -> Option<(Entity, TimeOfImpact)> {
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

impl CastRay<(Ray3d, ExcludeRigidBody)> for RapierContext<'_> {
	fn cast_ray(
		&self,
		(ray, ExcludeRigidBody(rigid_body)): &(Ray3d, ExcludeRigidBody),
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
