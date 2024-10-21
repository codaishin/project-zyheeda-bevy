use crate::traits::item_type::AssociatedItemType;
use bevy::prelude::*;
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Debug, PartialEq, Clone)]
pub struct Item<T>
where
	T: AssociatedItemType,
{
	pub name: &'static str,
	pub model: Option<&'static str>,
	pub content: Option<T>,
	pub item_type: T::TItemType,
}

impl<T> Default for Item<T>
where
	T: AssociatedItemType,
	T::TItemType: Default,
{
	fn default() -> Self {
		Self {
			name: default(),
			model: default(),
			content: default(),
			item_type: default(),
		}
	}
}

impl<T> Display for Item<T>
where
	T: AssociatedItemType,
{
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "Item({})", self.name)
	}
}
