use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::traits::accessors::get::Property;

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ItemType {
	#[default]
	Pistol,
	Bracer,
	ForceEssence,
	VoidBeam,
}

impl Property for ItemType {
	type TValue<'a> = Self;
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct CompatibleItems(pub HashSet<ItemType>);

impl<const N: usize> From<[ItemType; N]> for CompatibleItems {
	fn from(value: [ItemType; N]) -> Self {
		Self(HashSet::from(value))
	}
}

impl Property for CompatibleItems {
	type TValue<'a> = &'a HashSet<ItemType>;
}
