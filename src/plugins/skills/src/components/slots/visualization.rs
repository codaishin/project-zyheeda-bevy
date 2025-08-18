use bevy::prelude::*;
use std::{collections::HashMap, hash::Hash, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct SlotVisualization<TKey>
where
	TKey: Eq + Hash,
{
	pub(crate) slots: HashMap<TKey, Entity>,
	_p: PhantomData<TKey>,
}

impl<TKey> Default for SlotVisualization<TKey>
where
	TKey: Eq + Hash,
{
	fn default() -> Self {
		Self {
			slots: HashMap::default(),
			_p: PhantomData,
		}
	}
}

#[cfg(test)]
impl<T, TKey> From<T> for SlotVisualization<TKey>
where
	TKey: Eq + Hash,
	T: IntoIterator<Item = (TKey, Entity)>,
{
	fn from(value: T) -> Self {
		Self {
			slots: HashMap::from_iter(value),
			_p: PhantomData,
		}
	}
}
