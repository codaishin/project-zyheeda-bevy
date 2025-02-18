use common::tools::{item_type::ItemType, slot_key::SlotKey};

pub trait PeekNext {
	type TNext;

	fn peek_next(&self, trigger: &SlotKey, item_type: &ItemType) -> Option<Self::TNext>;
}
