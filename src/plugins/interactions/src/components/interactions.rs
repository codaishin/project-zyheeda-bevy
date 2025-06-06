use bevy::prelude::{Component, Entity, default};
use std::{collections::HashSet, fmt::Debug, marker::PhantomData};

#[derive(Component)]
pub(crate) struct Interactions<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	entities: HashSet<Entity>,
	_p: PhantomData<(TActor, TTarget)>,
}

impl<TActor, TTarget> Interactions<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	pub(crate) fn insert(&mut self, entity: Entity) -> bool {
		self.entities.insert(entity)
	}

	pub(crate) fn retain<F>(&mut self, predicate: F)
	where
		F: Fn(&Entity) -> bool,
	{
		self.entities.retain(predicate);
	}
}

impl<TActor, TTarget> Default for Interactions<TActor, TTarget>
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

impl<TActor, TTarget> Debug for Interactions<TActor, TTarget>
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

impl<TActor, TTarget> PartialEq for Interactions<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	fn eq(&self, other: &Self) -> bool {
		self._p == other._p && self.entities == other.entities
	}
}

#[cfg(test)]
impl<TActor, TTarget, const N: usize> From<[Entity; N]> for Interactions<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	fn from(entities: [Entity; N]) -> Self {
		Self {
			entities: HashSet::from(entities),
			_p: PhantomData,
		}
	}
}
