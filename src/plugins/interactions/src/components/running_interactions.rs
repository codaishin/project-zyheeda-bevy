use bevy::prelude::{Component, default};
use common::components::persistent_entity::PersistentEntity;
use std::{collections::HashSet, fmt::Debug, marker::PhantomData};

#[derive(Component)]
pub(crate) struct RunningInteractions<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	entities: HashSet<PersistentEntity>,
	_p: PhantomData<(TActor, TTarget)>,
}

impl<TActor, TTarget> RunningInteractions<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	pub(crate) fn insert(&mut self, entity: PersistentEntity) -> bool {
		self.entities.insert(entity)
	}

	pub(crate) fn retain<F>(&mut self, predicate: F)
	where
		F: FnMut(&PersistentEntity) -> bool,
	{
		self.entities.retain(predicate);
	}
}

impl<TActor, TTarget> Default for RunningInteractions<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	fn default() -> Self {
		Self {
			_p: PhantomData,
			entities: default(),
		}
	}
}

impl<TActor, TTarget> Debug for RunningInteractions<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Interactions")
			.field("entities", &self.entities)
			.field("_p", &self._p)
			.finish()
	}
}

impl<TActor, TTarget> PartialEq for RunningInteractions<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	fn eq(&self, other: &Self) -> bool {
		self._p == other._p && self.entities == other.entities
	}
}

#[cfg(test)]
impl<TActor, TTarget, const N: usize> From<[PersistentEntity; N]>
	for RunningInteractions<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	fn from(entities: [PersistentEntity; N]) -> Self {
		Self {
			entities: HashSet::from(entities),
			_p: PhantomData,
		}
	}
}
