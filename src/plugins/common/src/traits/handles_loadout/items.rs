use crate::traits::{
	accessors::get::{GetProperty, Property},
	handles_loadout::LoadoutKey,
	handles_localization::Token,
};
use std::ops::{Deref, DerefMut};

pub struct Items;

pub struct ItemToken;

impl Property for ItemToken {
	type TValue<'a> = &'a Token;
}

pub trait ReadItems {
	type TItem<'a>: GetProperty<ItemToken>
	where
		Self: 'a;

	fn get_item<TKey>(&self, key: TKey) -> Option<Self::TItem<'_>>
	where
		TKey: Into<LoadoutKey>;
}

impl<T> ReadItems for T
where
	T: Deref<Target: ReadItems>,
{
	type TItem<'a>
		= <<T as Deref>::Target as ReadItems>::TItem<'a>
	where
		Self: 'a;

	fn get_item<TKey>(&self, key: TKey) -> Option<Self::TItem<'_>>
	where
		TKey: Into<LoadoutKey>,
	{
		self.deref().get_item(key)
	}
}

pub trait SwapItems {
	fn swap_items<TA, TB>(&mut self, a: TA, b: TB)
	where
		TA: Into<LoadoutKey>,
		TB: Into<LoadoutKey>;
}

impl<T> SwapItems for T
where
	T: DerefMut<Target: SwapItems>,
{
	fn swap_items<TA, TB>(&mut self, a: TA, b: TB)
	where
		TA: Into<LoadoutKey>,
		TB: Into<LoadoutKey>,
	{
		self.deref_mut().swap_items(a, b);
	}
}
