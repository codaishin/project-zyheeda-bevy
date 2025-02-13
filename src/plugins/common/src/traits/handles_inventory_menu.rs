use bevy::prelude::*;
use std::{collections::HashMap, hash::Hash};

pub trait SwapKeys<TKey1, TKey2> {
	fn swap(&mut self, a: TKey1, b: TKey2);
}

pub trait GetDescriptor<TKey> {
	fn get_descriptor(&self, key: TKey) -> Option<&Descriptor>;
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Descriptor {
	pub name: String,
	pub icon: Option<Handle<Image>>,
}

// Needs to be moved to skills plugin
#[derive(Debug, PartialEq)]
pub struct Descriptions<TKey>(pub HashMap<TKey, Descriptor>)
where
	TKey: Eq + Hash;

impl<TKey> GetDescriptor<TKey> for Descriptions<TKey>
where
	TKey: Eq + Hash,
{
	fn get_descriptor(&self, key: TKey) -> Option<&Descriptor> {
		self.0.get(&key)
	}
}
