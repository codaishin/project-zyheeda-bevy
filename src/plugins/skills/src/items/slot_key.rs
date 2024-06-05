use common::{
	components::Side,
	traits::iteration::{Iter, IterKey},
};

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum SlotKey {
	Hand(Side),
}

impl Default for SlotKey {
	fn default() -> Self {
		Self::Hand(Side::Main)
	}
}

impl IterKey for SlotKey {
	fn iterator() -> Iter<Self> {
		Iter(Some(SlotKey::Hand(Side::Main)))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			SlotKey::Hand(Side::Main) => Some(SlotKey::Hand(Side::Off)),
			SlotKey::Hand(Side::Off) => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_all_keys() {
		assert_eq!(
			vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
			SlotKey::iterator().collect::<Vec<_>>()
		);
	}
}
