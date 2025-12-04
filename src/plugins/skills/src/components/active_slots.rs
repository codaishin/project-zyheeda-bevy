use bevy::prelude::*;
use common::tools::action_key::slot::SlotKey;
use std::{collections::HashSet, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct ActiveSlots<TFrame> {
	pub(crate) slots: HashSet<SlotKey>,
	_f: PhantomData<fn() -> TFrame>,
}

impl<T, TFrame> From<T> for ActiveSlots<TFrame>
where
	T: IntoIterator<Item = SlotKey>,
{
	fn from(slots: T) -> Self {
		Self {
			slots: HashSet::from_iter(slots),
			..default()
		}
	}
}

impl<TFrame> Default for ActiveSlots<TFrame> {
	fn default() -> Self {
		Self {
			slots: HashSet::default(),
			_f: PhantomData,
		}
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct Current;

#[derive(Debug, PartialEq)]
pub(crate) struct Old;
