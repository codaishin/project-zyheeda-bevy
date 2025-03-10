use super::{CastRay, TimeOfImpact};
use crate::tools::exclude_rigid_body::ExcludeRigidBody;
use bevy::{ecs::entity::Entity, math::Ray3d};
use bevy_rapier3d::{
	math::Real,
	pipeline::QueryFilterFlags,
	plugin::RapierContext,
	prelude::QueryFilter,
};

impl CastRay<Ray3d> for RapierContext {
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

impl CastRay<(Ray3d, ExcludeRigidBody)> for RapierContext {
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
