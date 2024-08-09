use bevy::prelude::default;
use std::{
	collections::{
		hash_map::{
			Entry as HashMapEntry,
			OccupiedEntry as HashMapOccupiedEntry,
			VacantEntry as HashMapVacantEntry,
		},
		HashMap,
	},
	hash::Hash,
};

/// A naive wrapper around a HashMap that retains key
/// insertion order by tracking it in a separate
/// vector containing copies of the respective key.
#[derive(Debug, PartialEq, Clone)]
pub struct OrderedHashMap<TKey, TValue>
where
	TKey: Eq + Hash,
{
	map: HashMap<TKey, TValue>,
	order: Vec<TKey>,
}

impl<TKey, TValue> OrderedHashMap<TKey, TValue>
where
	TKey: Eq + Hash + Copy,
{
	pub fn insert(&mut self, key: TKey, value: TValue) {
		self.order.retain(|pos| pos != &key);
		self.order.push(key);
		self.map.insert(key, value);
	}

	pub fn remove(&mut self, key: &TKey) -> Option<TValue> {
		self.order.retain(|pos| pos != key);
		self.map.remove(key)
	}

	pub fn get(&self, key: &TKey) -> Option<&TValue> {
		self.map.get(key)
	}

	pub fn get_mut(&mut self, key: &TKey) -> Option<&mut TValue> {
		self.map.get_mut(key)
	}

	pub fn entry(&mut self, key: TKey) -> Entry<TKey, TValue> {
		match self.map.entry(key) {
			HashMapEntry::Occupied(entry) => Entry::Occupied(OccupiedEntry { entry }),
			HashMapEntry::Vacant(entry) => Entry::Vacant(VacantEntry {
				entry,
				order: &mut self.order,
			}),
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = (&TKey, &TValue)> {
		self.order
			.iter()
			.filter_map(|key| Some((key, self.map.get(key)?)))
	}

	pub fn iter_mut(&mut self) -> impl Iterator<Item = (&TKey, &mut TValue)> {
		IterMut {
			map: &mut self.map,
			order: self.order.iter(),
		}
	}

	pub fn keys(&self) -> impl Iterator<Item = &TKey> {
		self.order.iter()
	}

	pub fn values(&self) -> impl Iterator<Item = &TValue> {
		self.order.iter().filter_map(|key| self.map.get(key))
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

pub struct IterMut<'a, TKey, TValue, TKeys>
where
	TKey: 'a + Eq + Hash,
	TKeys: Iterator<Item = &'a TKey>,
{
	map: &'a mut HashMap<TKey, TValue>,
	order: TKeys,
}

impl<'a, TKey, TValue, TKeys> Iterator for IterMut<'a, TKey, TValue, TKeys>
where
	TKey: Eq + Hash,
	TKeys: Iterator<Item = &'a TKey>,
{
	type Item = (&'a TKey, &'a mut TValue);

	fn next(&mut self) -> Option<Self::Item> {
		let key = self.order.next()?;
		let value = self.map.get_mut(key)?;

		// SAFETY: I believe this would work without `unsafe`,
		// if the method had a generic lifetime like
		// `fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>`.
		// However, as it stands the borrow checker doesn't seem
		// to be able to reason about that. But `self` outlives both
		// the `order` and the `map` field, so `unsafe` should be safe ;) here
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
		let key = self.0.order.remove(0);
		let value = self.0.map.remove(&key)?;
		Some((key, value))
	}
}

pub struct VacantEntry<'a, TKey, TValue>
where
	TKey: Eq + Hash + Copy,
{
	entry: HashMapVacantEntry<'a, TKey, TValue>,
	order: &'a mut Vec<TKey>,
}

impl<'a, TKey, TValue> VacantEntry<'a, TKey, TValue>
where
	TKey: Eq + Hash + Copy,
{
	pub fn insert(self, value: TValue) -> &'a mut TValue {
		self.order.push(*self.entry.key());
		self.entry.insert(value)
	}
}

pub struct OccupiedEntry<'a, TKey, TValue>
where
	TKey: Eq + Hash + Copy,
{
	entry: HashMapOccupiedEntry<'a, TKey, TValue>,
}

impl<'a, TKey, TValue> OccupiedEntry<'a, TKey, TValue>
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
}
