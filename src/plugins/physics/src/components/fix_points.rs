pub(crate) mod fix_point;

use crate::{
	components::{fix_points::fix_point::FixPointOf, mount_points::MountPoints},
	traits::query_filter_definition::QueryFilterDefinition,
};
use bevy::{
	ecs::entity::{EntityHashSet, hash_set::Iter},
	prelude::*,
};
use common::{
	components::persistent_entity::PersistentEntity,
	errors::{ErrorData, Level},
	tools::bone_name::BoneName,
	traits::{
		accessors::get::GetProperty,
		handles_skill_physics::SkillSpawner,
		thread_safe::ThreadSafe,
	},
};
use std::{any::type_name, collections::HashMap, fmt::Display, hash::Hash, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
#[require(Transform)]
pub(crate) struct Anchor<TFilter> {
	pub(crate) target: PersistentEntity,
	pub(crate) skill_spawner: SkillSpawner,
	pub(crate) use_target_rotation: bool,
	_p: PhantomData<fn() -> TFilter>,
}

impl QueryFilterDefinition for Anchor<Once> {
	type TFilter = Added<Self>;
}

impl QueryFilterDefinition for Anchor<Always> {
	type TFilter = ();
}

impl<TFilter> Anchor<TFilter>
where
	Self: QueryFilterDefinition + 'static,
{
	pub(crate) fn to_target<TEntity>(target: TEntity) -> AnchorBuilder<TFilter>
	where
		TEntity: Into<PersistentEntity>,
	{
		AnchorBuilder {
			target: target.into(),
			_p: PhantomData,
		}
	}

	pub(crate) fn with_target_rotation(mut self) -> Self {
		self.use_target_rotation = true;
		self
	}
}

impl<TFilter> GetProperty<PersistentEntity> for Anchor<TFilter> {
	fn get_property(&self) -> PersistentEntity {
		self.target
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Always;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Once;

pub(crate) struct AnchorBuilder<TFilter> {
	target: PersistentEntity,
	_p: PhantomData<TFilter>,
}

impl<TFilter> AnchorBuilder<TFilter> {
	pub(crate) fn on_spawner(self, spawner: SkillSpawner) -> Anchor<TFilter> {
		Anchor {
			target: self.target,
			skill_spawner: spawner,
			use_target_rotation: false,
			_p: PhantomData,
		}
	}
}

#[derive(Component, Debug, PartialEq, Clone, Default)]
#[relationship_target(relationship = FixPointOf)]
#[require(GlobalTransform)]
pub struct FixPoints(EntityHashSet);

impl FixPoints {
	pub(crate) fn iter(&self) -> Iter<'_> {
		self.0.iter()
	}
}

#[derive(Component, Debug, PartialEq, Clone)]
#[require(MountPoints<T>)]
pub struct MountPointsDefinition<T>(pub(crate) HashMap<BoneName, T>)
where
	T: Eq + Hash + ThreadSafe;

impl<T> Default for MountPointsDefinition<T>
where
	T: Eq + Hash + ThreadSafe,
{
	fn default() -> Self {
		Self(HashMap::default())
	}
}

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AnchorError {
	NoFixPointEntityFor(SkillSpawner),
	FixPointsMissingOn(PersistentEntity),
	GlobalTransformMissingOn(Entity),
	FixPointTranslationNaN(Entity),
}

impl Display for AnchorError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			AnchorError::FixPointsMissingOn(entity) => {
				let type_name = type_name::<FixPoints>();
				write!(f, "{entity:?}: {type_name} missing")
			}
			AnchorError::GlobalTransformMissingOn(entity) => {
				let type_name = type_name::<GlobalTransform>();
				write!(f, "{entity}: {type_name} missing")
			}
			AnchorError::NoFixPointEntityFor(entity) => {
				write!(f, "{entity:?} missing")
			}
			AnchorError::FixPointTranslationNaN(entity) => {
				write!(f, "{entity:?} translation is NaN")
			}
		}
	}
}

impl ErrorData for AnchorError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Anchor error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}
