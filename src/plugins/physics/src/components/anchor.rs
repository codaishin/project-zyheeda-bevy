use crate::traits::query_filter_definition::QueryFilterDefinition;
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{accessors::get::GetProperty, handles_skill_physics::SkillSpawner},
};
use std::marker::PhantomData;

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
