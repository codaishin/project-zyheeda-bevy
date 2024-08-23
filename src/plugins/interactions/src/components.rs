pub mod blocker;
pub mod interacting_entities;
pub mod is;

use crate::traits::ActOn;
use bevy::{
	ecs::{component::Component, entity::Entity},
	math::{Dir3, Vec3},
	utils::default,
};
use bevy_rapier3d::{
	geometry::CollisionGroups,
	pipeline::{QueryFilter, QueryFilterFlags},
};
use common::traits::cast_ray::TimeOfImpact;
use std::{marker::PhantomData, time::Duration};

#[derive(Component, Debug, PartialEq, Clone)]
pub struct RayCaster {
	pub origin: Vec3,
	pub direction: Dir3,
	pub max_toi: TimeOfImpact,
	pub solid: bool,
	pub filter: RayFilter,
}

impl Default for RayCaster {
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

#[derive(Default, Debug, PartialEq, Clone)]
pub struct RayFilter {
	flags: Option<QueryFilterFlags>,
	groups: Option<CollisionGroups>,
	exclude_collider: Option<Entity>,
	exclude_rigid_body: Option<Entity>,
}

#[derive(Debug, PartialEq)]
pub struct CannotParsePredicate;

impl<'a> TryFrom<QueryFilter<'a>> for RayFilter {
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

impl<'a> From<RayFilter> for QueryFilter<'a> {
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

#[derive(Component, Debug, PartialEq)]
pub enum Destroy {
	Immediately,
	AfterFrames(u8),
}

impl Destroy {
	pub const DELAYED: Destroy = Destroy::AfterFrames(2);
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct DealsDamage(pub i16);

#[derive(Component, Debug, PartialEq)]
pub struct Delay<TActor: ActOn<TTarget> + Clone, TTarget> {
	pub actor: TActor,
	pub after: Duration,
	pub(crate) timer: Duration,
	pub(crate) repeat: bool,
	phantom_data: PhantomData<TTarget>,
}

pub trait InitDelay<TTarget>
where
	Self: Clone + ActOn<TTarget>,
{
	fn after(self, duration: Duration) -> Delay<Self, TTarget>;
}

impl<TActor: Clone + ActOn<TTarget>, TTarget> InitDelay<TTarget> for TActor {
	fn after(self, duration: Duration) -> Delay<Self, TTarget> {
		Delay {
			actor: self,
			after: duration,
			timer: duration,
			repeat: false,
			phantom_data: PhantomData,
		}
	}
}

pub trait Repeat {
	fn repeat(self) -> Self;
}

impl<TActor: Clone + ActOn<TTarget>, TTarget> Repeat for Delay<TActor, TTarget> {
	fn repeat(self) -> Self {
		Self {
			repeat: true,
			actor: self.actor,
			after: self.after,
			timer: self.timer,
			phantom_data: self.phantom_data,
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
