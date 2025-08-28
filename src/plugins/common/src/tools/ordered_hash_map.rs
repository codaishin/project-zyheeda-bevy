use bevy::prelude::default;
use std::{
	collections::{
		HashMap,
		hash_map::{
			Entry as HashMapEntry,
			OccupiedEntry as HashMapOccupiedEntry,
			VacantEntry as HashMapVacantEntry,
		},
	},
	hash::Hash,
};

/// A naive wrapper around a HashMap that retains key
/// insertion order by tracking it in a separate
/// vector containing copies of the respective keys.
///
/// Removal and Insertion are `O(n)` operations.
#[derive(Debug, PartialEq, Clone)]
pub struct OrderedHashMap<TKey, TValue>
where
	TKey: Eq + Hash,
{
	map: HashMap<TKey, TValue>,
	order: keys::Unique<TKey>,
}

impl<TKey, TValue> OrderedHashMap<TKey, TValue>
where
	TKey: Eq + Hash + Copy,
{
	/// Inserts a key-value pair into the map.
	///
	/// Will cause this key to be last in the insertion order, even if the
	/// key-value pair was already contained.
	pub fn insert(&mut self, key: TKey, value: TValue) {
		self.order.push_back_unique(key);
		self.map.insert(key, value);
	}

	pub fn remove(&mut self, key: &TKey) -> Option<TValue> {
		self.order.remove(key);
		self.map.remove(key)
	}

	pub fn get(&self, key: &TKey) -> Option<&TValue> {
		self.map.get(key)
	}

	pub fn get_mut(&mut self, key: &TKey) -> Option<&mut TValue> {
		self.map.get_mut(key)
	}

	pub fn entry(&mut self, key: TKey) -> Entry<'_, TKey, TValue> {
		match self.map.entry(key) {
			HashMapEntry::Occupied(entry) => Entry::Occupied(OccupiedEntry { entry }),
			HashMapEntry::Vacant(entry) => Entry::Vacant(VacantEntry {
				entry,
				order: &mut self.order,
			}),
		}
	}

	pub fn iter(&self) -> Iter<'_, TKey, TValue> {
		Iter {
			map: &self.map,
			order: self.order.iter(),
		}
	}

	pub fn iter_mut(&mut self) -> IterMut<'_, TKey, TValue> {
		IterMut {
			map: &mut self.map,
			order: self.order.iter(),
		}
	}

	pub fn keys(&self) -> std::slice::Iter<'_, TKey> {
		self.order.iter()
	}

	pub fn values(&self) -> Values<'_, TKey, TValue> {
		Values {
			map: &self.map,
			order: self.order.iter(),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.map.is_empty()
	}
}

impl<TKey, TValue> Default for OrderedHashMap<TKey, TValue>
where
	TKey: Eq + Hash,
{
	fn default() -> Self {
		Self {
			map: default(),
			order: default(),
		}
	}
}

impl<const N: usize, TKey, TValue> From<[(TKey, TValue); N]> for OrderedHashMap<TKey, TValue>
where
	TKey: Eq + Hash + Copy,
{
	fn from(value: [(TKey, TValue); N]) -> Self {
		let mut map = Self::default();

		for (key, value) in value {
			map.insert(key, value);
		}

		map
	}
}

impl<TKey, TValue> FromIterator<(TKey, TValue)> for OrderedHashMap<TKey, TValue>
where
	TKey: Eq + Hash + Copy,
{
	fn from_iter<T: IntoIterator<Item = (TKey, TValue)>>(iter: T) -> Self {
		let mut map = Self::default();

		for (key, value) in iter {
			map.insert(key, value);
		}

		map
	}
}

pub struct Iter<'a, TKey, TValue> {
	map: &'a HashMap<TKey, TValue>,
	order: std::slice::Iter<'a, TKey>,
}

impl<'a, TKey, TValue> Iterator for Iter<'a, TKey, TValue>
where
	TKey: Eq + Hash,
{
	type Item = (&'a TKey, &'a TValue);

	fn next(&mut self) -> Option<Self::Item> {
		let key = self.order.next()?;
		let value = self.map.get(key)?;

		Some((key, value))
	}
}

pub struct IterMut<'a, TKey, TValue>
where
	TKey: 'a + Eq + Hash,
{
	map: &'a mut HashMap<TKey, TValue>,
	order: std::slice::Iter<'a, TKey>,
}

impl<'a, TKey, TValue> Iterator for IterMut<'a, TKey, TValue>
where
	TKey: Eq + Hash,
{
	type Item = (&'a TKey, &'a mut TValue);

	fn next(&mut self) -> Option<Self::Item> {
		let key = self.order.next()?;
		let value = self.map.get_mut(key)?;

		// Safety: The compiler cannot reason about disjointed mutable references here.
		// We are fine as long as `order` produces unique keys.
		// A streaming iterator - that uses `&'a mut self` - wouldn't have this problem.
		// It would tie each yielded value to the lifetime of the iterator not the implicit
		// lifetime of the function call.
		Some((key, unsafe { &mut *(value as *mut TValue) }))
	}
}

impl<TKey, TValue> IntoIterator for OrderedHashMap<TKey, TValue>
where
	TKey: Eq + Hash,
{
	type Item = (TKey, TValue);

	type IntoIter = IntoIter<TKey, TValue>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter(self)
	}
}

pub struct IntoIter<TKey, TValue>(OrderedHashMap<TKey, TValue>)
where
	TKey: Eq + Hash;

impl<TKey, TValue> Iterator for IntoIter<TKey, TValue>
where
	TKey: Eq + Hash,
{
	type Item = (TKey, TValue);

	fn next(&mut self) -> Option<Self::Item> {
		if self.0.order.is_empty() {
			return None;
		}
		let key = self.0.order.pop_front();
		let value = self.0.map.remove(&key)?;
		Some((key, value))
	}
}

pub struct Values<'a, TKey, TValue>
where
	TKey: Eq + Hash,
{
	map: &'a HashMap<TKey, TValue>,
	order: std::slice::Iter<'a, TKey>,
}

impl<'a, TKey, TValue> Iterator for Values<'a, TKey, TValue>
where
	TKey: Eq + Hash,
{
	type Item = &'a TValue;

	fn next(&mut self) -> Option<Self::Item> {
		let key = self.order.next()?;
		let value = self.map.get(key)?;

		Some(value)
	}
}

pub struct VacantEntry<'a, TKey, TValue>
where
	TKey: Eq + Hash + Copy,
{
	entry: HashMapVacantEntry<'a, TKey, TValue>,
	order: &'a mut keys::Unique<TKey>,
}

impl<'a, TKey, TValue> VacantEntry<'a, TKey, TValue>
where
	TKey: Eq + Hash + Copy,
{
	pub fn insert(self, value: TValue) -> &'a mut TValue {
		self.order.push_back_unique(*self.entry.key());
		self.entry.insert(value)
	}
}

pub struct OccupiedEntry<'a, TKey, TValue>
where
	TKey: Eq + Hash + Copy,
{
	entry: HashMapOccupiedEntry<'a, TKey, TValue>,
}

impl<TKey, TValue> OccupiedEntry<'_, TKey, TValue>
where
	TKey: Eq + Hash + Copy,
{
	pub fn get(&self) -> &TValue {
		self.entry.get()
	}

	pub fn get_mut(&mut self) -> &mut TValue {
		self.entry.get_mut()
	}
}

pub enum Entry<'a, TKey, TValue>
where
	TKey: Eq + Hash + Copy,
{
	Vacant(VacantEntry<'a, TKey, TValue>),
	Occupied(OccupiedEntry<'a, TKey, TValue>),
}

impl<TKey, TValue> Extend<(TKey, TValue)> for OrderedHashMap<TKey, TValue>
where
	TKey: Eq + Hash + Copy,
{
	fn extend<T: IntoIterator<Item = (TKey, TValue)>>(&mut self, iter: T) {
		for (key, value) in iter {
			self.insert(key, value);
		}
	}
}

mod keys {

	/// Holds unique values like a [`HashSet`](std::collections::HashSet),
	/// but retains insertion order.
	///
	/// Removal and Insertion are `O(n)` operations.
	#[derive(Debug, PartialEq, Clone)]
	pub(super) struct Unique<TKey>(Vec<TKey>)
	where
		TKey: PartialEq;

	impl<TKey> Unique<TKey>
	where
		TKey: PartialEq,
	{
		pub(super) fn is_empty(&self) -> bool {
			self.0.is_empty()
		}

		pub(super) fn push_back_unique(&mut self, key: TKey) {
			self.remove(&key);
			self.0.push(key);
		}

		pub(super) fn remove(&mut self, key: &TKey) {
			self.0.retain(|k| k != key);
		}

		pub(super) fn pop_front(&mut self) -> TKey {
			self.0.remove(0)
		}

		pub(super) fn iter(&self) -> std::slice::Iter<'_, TKey> {
			self.0.iter()
		}
	}

	impl<TKey> Default for Unique<TKey>
	where
		TKey: PartialEq,
	{
		fn default() -> Self {
			Self(Vec::default())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	macro_rules! repeat {
		($count:expr, $code:expr) => {
			for _ in 0..$count {
				$code
			}
		};
	}

	#[test]
	fn insert_value_and_iterate_in_order() {
		repeat!(100, {
			let mut map = OrderedHashMap::<&'static str, u32>::default();

			map.insert("first", 0);
			map.insert("second", 1);

			assert_eq!(
				vec![(&"first", &0), (&"second", &1)],
				map.iter().collect::<Vec<_>>()
			)
		})
	}

	#[test]
	fn insert_value_and_move_iterate_in_order() {
		repeat!(100, {
			let mut map = OrderedHashMap::<&'static str, u32>::default();

			map.insert("first", 0);
			map.insert("second", 1);

			assert_eq!(
				vec![("first", 0), ("second", 1)],
				map.into_iter().collect::<Vec<_>>()
			)
		})
	}

	#[test]
	fn insert_value_and_iterate_values_in_order() {
		repeat!(100, {
			let mut map = OrderedHashMap::<&'static str, u32>::default();

			map.insert("first", 0);
			map.insert("second", 1);

			assert_eq!(vec![&0, &1], map.values().collect::<Vec<_>>())
		})
	}

	#[test]
	fn insert_value_and_iterate_keys_in_order() {
		repeat!(100, {
			let mut map = OrderedHashMap::<&'static str, u32>::default();

			map.insert("first", 0);
			map.insert("second", 1);

			assert_eq!(vec![&"first", &"second"], map.keys().collect::<Vec<_>>())
		})
	}

	#[test]
	fn insert_duplicate_key_pushes_that_item_back_for_iteration() {
		let mut map = OrderedHashMap::<&'static str, u32>::default();

		map.insert("first", 0);
		map.insert("second", 1);
		map.insert("first", 2);

		assert_eq!(
			vec![(&"second", &1), (&"first", &2),],
			map.iter().collect::<Vec<_>>()
		)
	}

	#[test]
	fn get_inserted_value() {
		let mut map = OrderedHashMap::<&'static str, u32>::default();

		map.insert("first", 0);

		assert_eq!(Some(&0), map.get(&"first"));
	}

	#[test]
	fn get_mut_inserted_value() {
		let mut map = OrderedHashMap::<&'static str, u32>::default();

		map.insert("first", 0);

		assert_eq!(Some(&mut 0), map.get_mut(&"first"));
	}

	#[test]
	fn remove_gets_value() {
		let mut map = OrderedHashMap::<&'static str, u32>::default();

		map.insert("first", 0);

		assert_eq!(Some(0), map.remove(&"first"));
	}

	#[test]
	fn remove_removes_value() {
		let mut map = OrderedHashMap::<&'static str, u32>::default();

		map.insert("first", 0);
		map.remove(&"first");

		assert_eq!(None, map.get(&"first"));
	}

	#[test]
	fn remove_value_from_order() {
		let mut map = OrderedHashMap::<&'static str, u32>::default();

		map.insert("first", 0);
		map.remove(&"first");

		assert_eq!(
			vec![] as Vec<(&&'static str, &u32)>,
			map.iter().collect::<Vec<_>>()
		)
	}

	#[test]
	fn remove_non_first_value_from_order() {
		let mut map = OrderedHashMap::<&'static str, u32>::default();

		map.insert("first", 0);
		map.insert("second", 1);
		map.remove(&"second");

		assert_eq!(vec![(&"first", &0)], map.iter().collect::<Vec<_>>())
	}

	#[test]
	fn get_inserted_entry() {
		let mut map = OrderedHashMap::<&'static str, u32>::default();

		map.insert("first", 0);

		let Entry::Occupied(entry) = map.entry("first") else {
			panic!("Expected a occupied entry, but it was vacant");
		};

		assert_eq!(&0, entry.get());
	}

	#[test]
	fn get_mut_inserted_entry() {
		let mut map = OrderedHashMap::<&'static str, u32>::default();

		map.insert("first", 0);

		let Entry::Occupied(mut entry) = map.entry("first") else {
			panic!("Expected a occupied entry, but it was vacant");
		};

		assert_eq!(&mut 0, entry.get_mut());
	}

	#[test]
	fn update_order_when_inserting_on_empty_value() {
		let mut map = OrderedHashMap::<&'static str, u32>::default();

		map.insert("first", 0);

		let Entry::Vacant(entry) = map.entry("second") else {
			panic!("Expected a vacant entry, but it was occupied");
		};

		entry.insert(1);

		assert_eq!(
			vec![(&"first", &0), (&"second", &1)],
			map.iter().collect::<Vec<_>>()
		);
	}

	#[test]
	fn extend() {
		repeat!(100, {
			let mut map = OrderedHashMap::<&'static str, u32>::default();

			map.insert("first", 0);
			map.extend([("second", 1)].into_iter());

			assert_eq!(
				vec![(&"first", &0), (&"second", &1)],
				map.iter().collect::<Vec<_>>()
			)
		})
	}

	#[test]
	fn from_array_empty() {
		assert_eq!(
			OrderedHashMap::<&'static str, u32>::default(),
			OrderedHashMap::<&'static str, u32>::from([])
		)
	}

	#[test]
	fn from_array_ordered() {
		let mut expected = OrderedHashMap::<&'static str, u32>::default();
		expected.insert("first", 0);
		expected.insert("second", 1);

		assert_eq!(
			expected,
			OrderedHashMap::from([("first", 0), ("second", 1)])
		)
	}

	#[test]
	fn from_iter_empty() {
		assert_eq!(
			OrderedHashMap::<&'static str, u32>::default(),
			OrderedHashMap::<&'static str, u32>::from_iter([].into_iter())
		)
	}

	#[test]
	fn from_iter_ordered() {
		let mut expected = OrderedHashMap::<&'static str, u32>::default();
		expected.insert("first", 0);
		expected.insert("second", 1);

		assert_eq!(
			expected,
			OrderedHashMap::from_iter([("first", 0), ("second", 1)].into_iter())
		)
	}

	#[test]
	fn iter_mut() {
		repeat!(100, {
			let mut map =
				OrderedHashMap::from([("first", "0".to_owned()), ("second", "1".to_owned())]);
			let mut order = vec![];

			for (key, value) in map.iter_mut() {
				*value = format!("{}: {}", key, *value);
				order.push(*key);
			}

			assert_eq!(
				(
					vec!["first", "second"],
					OrderedHashMap::from_iter([
						("first", "first: 0".to_owned()),
						("second", "second: 1".to_owned())
					])
				),
				(order, map),
			)
		})
	}

	#[test]
	fn keys() {
		let map = OrderedHashMap::from([(-11, "0"), (32, "1")]);

		assert_eq!(vec![&-11, &32], map.keys().collect::<Vec<_>>());
	}

	#[test]
	fn values() {
		let map = OrderedHashMap::from([(-11, "0"), (32, "1")]);

		assert_eq!(vec![&"0", &"1"], map.values().collect::<Vec<_>>());
	}

	#[test]
	fn is_empty() {
		let filled = OrderedHashMap::from([(32, 32)]);
		let empty = OrderedHashMap::<u32, u32>::from([]);

		assert_eq!((false, true), (filled.is_empty(), empty.is_empty()));
	}
}
