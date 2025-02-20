pub mod key_map;
pub mod language_server;

use bevy::prelude::*;
use std::{collections::HashMap, fmt::Debug, hash::Hash};

#[derive(Resource)]
pub struct Shared<TKey: Eq + Hash, T: Clone> {
	map: HashMap<TKey, T>,
}

impl<TKey: Eq + Hash + Debug, T: Clone + Debug> Debug for Shared<TKey, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Shared").field("map", &self.map).finish()
	}
}

impl<TKey: Eq + Hash + PartialEq, T: Clone + PartialEq> PartialEq for Shared<TKey, T> {
	fn eq(&self, other: &Self) -> bool {
		self.map == other.map
	}
}

impl<TKey: Eq + Hash, T: Clone> Shared<TKey, T> {
	pub fn new<const N: usize>(values: [(TKey, T); N]) -> Self {
		Self {
			map: HashMap::from(values),
		}
	}
}

impl<TKey: Eq + Hash, T: Clone> Default for Shared<TKey, T> {
	fn default() -> Self {
		Self {
			map: HashMap::default(),
		}
	}
}

impl<TKey: Eq + Hash, T: Clone> Shared<TKey, T> {
	pub fn get(&self, key: &TKey) -> Option<&T> {
		self.map.get(key)
	}
}

impl<TKey: Eq + Hash, T: Clone> From<HashMap<TKey, T>> for Shared<TKey, T> {
	fn from(map: HashMap<TKey, T>) -> Self {
		Self { map }
	}
}
