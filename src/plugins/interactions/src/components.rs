use crate::traits::ActOn;
use bevy::{
	ecs::{component::Component, entity::Entity},
	math::Vec3,
};
use bevy_rapier3d::{
	geometry::CollisionGroups,
	pipeline::{QueryFilter, QueryFilterFlags},
};
use common::traits::cast_ray::TimeOfImpact;
use std::{collections::HashSet, marker::PhantomData, time::Duration};

#[derive(Component, Default, Debug, PartialEq, Clone)]
pub struct RayCaster {
	pub origin: Vec3,
	pub direction: Vec3,
	pub max_toi: TimeOfImpact,
	pub solid: bool,
	pub filter: RayFilter,
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct RayFilter(HashSet<FilterOptions>);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum FilterOptions {
	Flags(QueryFilterFlags),
	Groups(CollisionGroups),
	ExcludeCollider(Entity),
	ExcludeRigidBody(Entity),
}

#[derive(Debug, PartialEq)]
pub struct CannotParsePredicate;

impl<'a> TryFrom<QueryFilter<'a>> for RayFilter {
	type Error = CannotParsePredicate;

	fn try_from(query_filter: QueryFilter) -> Result<Self, CannotParsePredicate> {
		if query_filter.predicate.is_some() {
			return Err(CannotParsePredicate);
		}

		let mut f = HashSet::new();

		if !query_filter.flags.is_empty() {
			f.insert(FilterOptions::Flags(query_filter.flags));
		}
		if let Some(groups) = query_filter.groups {
			f.insert(FilterOptions::Groups(groups));
		}
		if let Some(entity) = query_filter.exclude_collider {
			f.insert(FilterOptions::ExcludeCollider(entity));
		}
		if let Some(entity) = query_filter.exclude_rigid_body {
			f.insert(FilterOptions::ExcludeRigidBody(entity));
		}

		Ok(RayFilter(f))
	}
}

impl<'a> From<RayFilter> for QueryFilter<'a> {
	fn from(ray_filter: RayFilter) -> Self {
		let mut f = QueryFilter::new();

		for option in ray_filter.0 {
			match option {
				FilterOptions::Flags(flags) => f.flags = flags,
				FilterOptions::Groups(groups) => f.groups = Some(groups),
				FilterOptions::ExcludeCollider(entity) => f.exclude_collider = Some(entity),
				FilterOptions::ExcludeRigidBody(entity) => f.exclude_rigid_body = Some(entity),
			}
		}

		f
	}
}

#[derive(Component)]
pub(crate) struct Destroy;

#[derive(Component, Clone)]
pub struct DealsDamage(pub i16);

#[derive(Component)]
pub struct Repeat<TActor: ActOn<TTarget> + Clone, TTarget> {
	pub actor: TActor,
	pub after: Duration,
	pub(crate) timer: Duration,
	phantom_data: PhantomData<TTarget>,
}

pub trait RepeatAfter<TTarget>
where
	Self: Clone + ActOn<TTarget>,
{
	fn repeat_after(self, duration: Duration) -> Repeat<Self, TTarget>;
}

impl<TActor: Clone + ActOn<TTarget>, TTarget> RepeatAfter<TTarget> for TActor {
	fn repeat_after(self, duration: Duration) -> Repeat<Self, TTarget> {
		Repeat {
			actor: self,
			after: duration,
			timer: duration,
			phantom_data: PhantomData,
		}
	}
}

#[cfg(test)]
mod tests_ray_filter_from_query_filter {
	use super::*;
	use bevy_rapier3d::geometry::Group;

	#[test]
	fn set_all_flags() {
		let ray_filter = RayFilter(HashSet::from([
			FilterOptions::Flags(QueryFilterFlags::EXCLUDE_FIXED),
			FilterOptions::Groups(CollisionGroups {
				memberships: Group::all(),
				filters: Group::empty(),
			}),
			FilterOptions::ExcludeCollider(Entity::from_raw(42)),
			FilterOptions::ExcludeRigidBody(Entity::from_raw(24)),
		]));
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
			),
			(
				query_filter.flags,
				query_filter.groups,
				query_filter.exclude_collider,
				query_filter.exclude_rigid_body
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
		let mut query_filter = QueryFilter::new();
		query_filter.flags = QueryFilterFlags::EXCLUDE_FIXED;
		query_filter.groups = Some(CollisionGroups {
			memberships: Group::all(),
			filters: Group::empty(),
		});
		query_filter.exclude_collider = Some(Entity::from_raw(42));
		query_filter.exclude_rigid_body = Some(Entity::from_raw(24));
		let ray_filter = RayFilter::try_from(query_filter);

		assert_eq!(
			Ok(RayFilter(HashSet::from([
				FilterOptions::Flags(QueryFilterFlags::EXCLUDE_FIXED),
				FilterOptions::Groups(CollisionGroups {
					memberships: Group::all(),
					filters: Group::empty(),
				}),
				FilterOptions::ExcludeCollider(Entity::from_raw(42)),
				FilterOptions::ExcludeRigidBody(Entity::from_raw(24)),
			]))),
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
