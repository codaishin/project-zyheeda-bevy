use common::tools::{item_type::ItemType, slot_key::SlotKey};

pub trait PeekNext<'a> {
	type TNext;

	fn peek_next(&'a self, trigger: &SlotKey, item_type: &ItemType) -> Option<&'a Self::TNext>;
}
