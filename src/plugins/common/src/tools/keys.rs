pub mod movement;
pub mod slot;

use crate::traits::{
	get_ui_text::{English, GetUiText, Japanese, UIText},
	iteration::{Iter, IterFinite},
};
use bevy::utils::default;
use movement::MovementKey;
use slot::SlotKey;

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum Key {
	Movement(MovementKey),
	Slot(SlotKey),
}

impl Default for Key {
	fn default() -> Self {
		Self::Movement(default())
	}
}

impl IterFinite for Key {
	fn iterator() -> Iter<Self> {
		Iter(Some(Self::default()))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			Key::Movement(key) => try_next(Key::Movement, key).or(try_fst(Key::Slot)),
			Key::Slot(key) => try_next(Key::Slot, key),
		}
	}
}

impl GetUiText<Key> for English {
	fn ui_text(key: &Key) -> UIText {
		match key {
			Key::Movement(key) => English::ui_text(key),
			Key::Slot(key) => English::ui_text(key),
		}
	}
}

impl GetUiText<Key> for Japanese {
	fn ui_text(key: &Key) -> UIText {
		match key {
			Key::Movement(key) => Japanese::ui_text(key),
			Key::Slot(key) => Japanese::ui_text(key),
		}
	}
}

fn try_next<TInner>(wrap: impl Fn(TInner) -> Key, key: TInner) -> Option<Key>
where
	TInner: IterFinite,
{
	TInner::next(&Iter(Some(key))).map(wrap)
}

fn try_fst<TInner>(wrap: impl Fn(TInner) -> Key) -> Option<Key>
where
	TInner: IterFinite,
{
	TInner::iterator().0.map(wrap)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::iteration::IterFinite;

	#[test]
	fn iter_all_keys() {
		assert_eq!(
			MovementKey::iterator()
				.map(Key::Movement)
				.chain(SlotKey::iterator().map(Key::Slot))
				.collect::<Vec<_>>(),
			Key::iterator().take(100).collect::<Vec<_>>()
		);
	}
}
