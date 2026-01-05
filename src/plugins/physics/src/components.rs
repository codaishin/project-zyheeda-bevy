pub(crate) mod active_beam;
pub(crate) mod affected;
pub(crate) mod blockable;
pub(crate) mod blocker_types;
pub(crate) mod colliders;
pub(crate) mod default_attributes;
pub(crate) mod effect;
pub(crate) mod fix_points;
pub(crate) mod ground_target;
pub(crate) mod hollow;
pub(crate) mod interacting_entities;
pub(crate) mod interaction_target;
pub(crate) mod motion;
pub(crate) mod no_hover;
pub(crate) mod running_interactions;
pub(crate) mod set_motion_forward;
pub(crate) mod skill_prefabs;
pub(crate) mod when_traveled;
pub(crate) mod world_camera;

use bevy::{
	ecs::{component::Component, entity::Entity},
	math::{Dir3, Vec3},
	utils::default,
};
use bevy_rapier3d::{
	geometry::CollisionGroups,
	pipeline::{QueryFilter, QueryFilterFlags},
};

use common::traits::handles_physics::TimeOfImpact;
#[cfg(test)]
use testing::ApproxEqual;

#[derive(Component, Debug, PartialEq, Clone)]
pub struct RayCasterArgs {
	pub origin: Vec3,
	pub direction: Dir3,
	pub max_toi: TimeOfImpact,
	pub solid: bool,
	pub filter: RayFilter,
}

impl Default for RayCasterArgs {
	fn default() -> Self {
		Self {
			origin: Default::default(),
			direction: Dir3::NEG_Z,
			max_toi: Default::default(),
			solid: Default::default(),
			filter: Default::default(),
		}
	}
}

#[cfg(test)]
impl ApproxEqual<f32> for RayCasterArgs {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		self.origin.approx_equal(&other.origin, tolerance)
			&& self.direction.approx_equal(&other.direction, tolerance)
			&& self.max_toi.approx_equal(&other.max_toi, tolerance)
			&& self.solid == other.solid
			&& self.filter == other.filter
	}
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct RayFilter {
	flags: Option<QueryFilterFlags>,
	groups: Option<CollisionGroups>,
	exclude_collider: Option<Entity>,
	exclude_rigid_body: Option<Entity>,
}

#[derive(Debug, PartialEq)]
pub struct CannotParsePredicate;

impl TryFrom<QueryFilter<'_>> for RayFilter {
	type Error = CannotParsePredicate;

	fn try_from(query_filter: QueryFilter) -> Result<Self, CannotParsePredicate> {
		if query_filter.predicate.is_some() {
			return Err(CannotParsePredicate);
		}

		let mut filter = RayFilter::default();

		if !query_filter.flags.is_empty() {
			filter.flags = Some(query_filter.flags);
		}
		if let Some(groups) = query_filter.groups {
			filter.groups = Some(groups);
		}
		if let Some(entity) = query_filter.exclude_collider {
			filter.exclude_collider = Some(entity);
		}
		if let Some(entity) = query_filter.exclude_rigid_body {
			filter.exclude_rigid_body = Some(entity);
		}

		Ok(filter)
	}
}

impl From<RayFilter> for QueryFilter<'_> {
	fn from(ray_filter: RayFilter) -> Self {
		Self {
			groups: ray_filter.groups,
			exclude_collider: ray_filter.exclude_collider,
			exclude_rigid_body: ray_filter.exclude_rigid_body,
			flags: ray_filter.flags.unwrap_or_default(),
			..default()
		}
	}
}

#[cfg(test)]
mod tests_ray_filter_from_query_filter {
	use super::*;
	use bevy_rapier3d::geometry::Group;

	#[test]
	fn set_all_flags() {
		let ray_filter = RayFilter {
			flags: Some(QueryFilterFlags::EXCLUDE_FIXED),
			groups: Some(CollisionGroups {
				memberships: Group::all(),
				filters: Group::empty(),
			}),
			exclude_collider: Some(Entity::from_raw(42)),
			exclude_rigid_body: Some(Entity::from_raw(24)),
		};
		let query_filter: QueryFilter = ray_filter.into();

		assert_eq!(
			(
				QueryFilterFlags::EXCLUDE_FIXED,
				Some(CollisionGroups {
					memberships: Group::all(),
					filters: Group::empty(),
				}),
				Some(Entity::from_raw(42)),
				Some(Entity::from_raw(24)),
				true,
			),
			(
				query_filter.flags,
				query_filter.groups,
				query_filter.exclude_collider,
				query_filter.exclude_rigid_body,
				query_filter.predicate.is_none(),
			)
		)
	}
}

#[cfg(test)]
mod test_ray_filter_from_query_filter {
	use super::*;
	use bevy_rapier3d::geometry::Group;

	#[test]
	fn set_all_flags_except_dynamic_filter() {
		let query_filter = QueryFilter {
			flags: QueryFilterFlags::EXCLUDE_FIXED,
			groups: Some(CollisionGroups {
				memberships: Group::all(),
				filters: Group::empty(),
			}),
			exclude_collider: Some(Entity::from_raw(42)),
			exclude_rigid_body: Some(Entity::from_raw(24)),
			..default()
		};
		let ray_filter = RayFilter::try_from(query_filter);

		assert_eq!(
			Ok(RayFilter {
				flags: Some(QueryFilterFlags::EXCLUDE_FIXED),
				groups: Some(CollisionGroups {
					memberships: Group::all(),
					filters: Group::empty(),
				}),
				exclude_collider: Some(Entity::from_raw(42)),
				exclude_rigid_body: Some(Entity::from_raw(24)),
			}),
			ray_filter
		);
	}

	#[test]
	fn predicate_error() {
		let mut query_filter = QueryFilter::new();
		query_filter.predicate = Some(&|_| false);

		let ray_filter = RayFilter::try_from(query_filter);

		assert_eq!(Err(CannotParsePredicate), ray_filter);
	}
}
