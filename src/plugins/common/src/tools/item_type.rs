use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ItemType {
	#[default]
	Pistol,
	Bracer,
	ForceEssence,
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct CompatibleItems(pub HashSet<ItemType>);

impl<const N: usize> From<[ItemType; N]> for CompatibleItems {
	fn from(value: [ItemType; N]) -> Self {
		Self(HashSet::from(value))
	}
}
