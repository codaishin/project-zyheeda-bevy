use bevy::prelude::*;
use common::tools::action_key::slot::SlotKey;
use std::{
	collections::{HashSet, hash_set::Iter},
	ops::{Deref, DerefMut},
};

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct HeldSlots {
	current: HashSet<SlotKey>,
	previous: HashSet<SlotKey>,
}

impl HeldSlots {
	/// Rotate current to old keys.
	pub(crate) fn rotate(&mut self) {
		std::mem::swap(&mut self.previous, &mut self.current);

		self.current.clear();
	}

	pub(crate) fn iter_new(&self) -> IterNew<'_> {
		IterNew {
			previous: &self.previous,
			current: self.current.iter(),
		}
	}
}

impl<const N: usize> From<[SlotKey; N]> for HeldSlots {
	fn from(keys: [SlotKey; N]) -> Self {
		Self {
			current: HashSet::from(keys),
			..default()
		}
	}
}

impl Deref for HeldSlots {
	type Target = HashSet<SlotKey>;

	fn deref(&self) -> &Self::Target {
		&self.current
	}
}

impl DerefMut for HeldSlots {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.current
	}
}

pub(crate) struct IterNew<'a> {
	previous: &'a HashSet<SlotKey>,
	current: Iter<'a, SlotKey>,
}

impl<'a> Iterator for IterNew<'a> {
	type Item = &'a SlotKey;

	fn next(&mut self) -> Option<Self::Item> {
		for current in &mut self.current {
			if self.previous.contains(current) {
				continue;
			}

			return Some(current);
		}

		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn deref() {
		let slots = HeldSlots {
			current: HashSet::from([SlotKey(42), SlotKey(11)]),
			..default()
		};

		assert_eq!(HashSet::from([SlotKey(42), SlotKey(11)]), *slots);
	}

	#[test]
	fn deref_mut() {
		let mut slots = HeldSlots::default();

		*slots = HashSet::from([SlotKey(42), SlotKey(11)]);

		assert_eq!(HashSet::from([SlotKey(42), SlotKey(11)]), slots.current);
	}

	#[test]
	fn rotate() {
		let mut slots = HeldSlots {
			current: HashSet::from([SlotKey(42), SlotKey(11)]),
			previous: HashSet::from([SlotKey(100)]),
		};

		slots.rotate();

		assert_eq!(
			HeldSlots {
				current: HashSet::from([]),
				previous: HashSet::from([SlotKey(42), SlotKey(11)]),
			},
			slots
		);
	}

	#[test]
	fn iter_new() {
		let slots = HeldSlots {
			current: HashSet::from([SlotKey(42), SlotKey(11), SlotKey(110), SlotKey(3)]),
			previous: HashSet::from([SlotKey(42), SlotKey(110)]),
		};

		assert_eq!(
			HashSet::from([&SlotKey(11), &SlotKey(3)]),
			slots.iter_new().collect::<HashSet<_>>(),
		);
	}
}
