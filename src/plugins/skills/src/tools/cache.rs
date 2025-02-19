use common::traits::handles_loadout_menu::GetItem;
use std::{collections::HashMap, hash::Hash};

#[derive(Debug, PartialEq)]
pub struct Cache<TKey, TItem>(pub HashMap<TKey, TItem>)
where
	TKey: Eq + Hash;

impl<TKey, TItem> GetItem<TKey> for Cache<TKey, TItem>
where
	TKey: Eq + Hash,
{
	type TItem = TItem;

	fn get_item(&self, key: TKey) -> Option<&TItem> {
		self.0.get(&key)
	}
}
