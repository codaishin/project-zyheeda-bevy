use crate::traits::accessors::get::ViewField;
use macros::serde_model;
use std::collections::HashSet;

#[serde_model]
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ItemType {
	#[default]
	Pistol,
	Bracer,
	ForceEssence,
	VoidBeam,
}

impl ViewField for ItemType {
	type TValue<'a> = Self;
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct CompatibleItems(pub HashSet<ItemType>);

impl<const N: usize> From<[ItemType; N]> for CompatibleItems {
	fn from(value: [ItemType; N]) -> Self {
		Self(HashSet::from(value))
	}
}

impl ViewField for CompatibleItems {
	type TValue<'a> = &'a HashSet<ItemType>;
}
