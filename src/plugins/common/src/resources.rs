pub mod key_map;
pub mod language_server;

use crate::traits::cache::Storage;
use bevy::prelude::*;
use std::{
	collections::{
		hash_map::Entry::{Occupied, Vacant},
		HashMap,
	},
	fmt::Debug,
	hash::Hash,
};

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

impl<TKey: Eq + Hash, T: Clone> Storage<TKey, T> for Shared<TKey, T> {
	fn get_or_create(&mut self, key: TKey, create: impl FnOnce() -> T) -> T {
		match self.map.entry(key) {
			Occupied(entry) => entry.get().clone(),
			Vacant(entry) => entry.insert(create()).clone(),
		}
	}
}

impl<TKey: Eq + Hash, T: Clone> From<HashMap<TKey, T>> for Shared<TKey, T> {
	fn from(map: HashMap<TKey, T>) -> Self {
		Self { map }
	}
}

#[cfg(test)]
mod test_shared_asset {
	use super::*;
	use uuid::Uuid;

	#[test]
	fn get_new() {
		let mut called = false;
		let new_handle = Handle::<Mesh>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut shared_assets = Shared::<u32, Handle<Mesh>>::default();
		let handle = shared_assets.get_or_create(42, || {
			called = true;
			new_handle.clone()
		});

		assert_eq!((new_handle, true), (handle, called));
	}

	#[test]
	fn get_registered() {
		let mut called = false;
		let old_handle = Handle::<Mesh>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut shared_assets = Shared::<u32, Handle<Mesh>>::default();
		_ = shared_assets.get_or_create(42, || old_handle.clone());
		let handle = shared_assets.get_or_create(42, || {
			called = true;
			Handle::default()
		});

		assert_eq!((old_handle, false), (handle, called));
	}

	#[test]
	fn get_new_on_different_key() {
		let mut called = false;
		let new_handle = Handle::<Mesh>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut shared_assets = Shared::<u32, Handle<Mesh>>::default();
		_ = shared_assets.get_or_create(42, Handle::default);
		let handle = shared_assets.get_or_create(43, || {
			called = true;
			new_handle.clone()
		});

		assert_eq!((new_handle, true), (handle, called));
	}
}
