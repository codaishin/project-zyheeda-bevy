use bevy::prelude::*;
use std::{collections::HashMap, hash::Hash};

use super::accessors::get::Getter;

pub trait SwapKeys<TKey1, TKey2> {
	fn swap(&mut self, a: TKey1, b: TKey2);
}

pub trait GetDescriptor<TKey> {
	type TItem;

	fn get_descriptor(&self, key: TKey) -> Option<&Self::TItem>;
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct InventoryDescriptor {
	pub name: String,
	pub icon: Option<Handle<Image>>,
}

impl Getter<Name> for InventoryDescriptor {
	fn get(&self) -> Name {
		Name::from(self.name.clone())
	}
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct QuickbarDescriptor {
	pub name: String,
	pub execution: SkillExecution,
	pub icon: Option<Handle<Image>>,
}

impl Getter<Name> for QuickbarDescriptor {
	fn get(&self) -> Name {
		Name::from(self.name.clone())
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub enum SkillExecution {
	#[default]
	None,
	Active,
	Queued,
}

// Needs to be moved to skills plugin
#[derive(Debug, PartialEq)]
pub struct Descriptions<TKey, TItem>(pub HashMap<TKey, TItem>)
where
	TKey: Eq + Hash;

impl<TKey, TItem> GetDescriptor<TKey> for Descriptions<TKey, TItem>
where
	TKey: Eq + Hash,
{
	type TItem = TItem;

	fn get_descriptor(&self, key: TKey) -> Option<&TItem> {
		self.0.get(&key)
	}
}
