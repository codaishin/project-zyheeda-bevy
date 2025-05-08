use common::tools::{action_key::slot::SlotKey, item_type::ItemType};

pub trait PeekNext<'a> {
	type TNext;

	fn peek_next(&'a self, trigger: &SlotKey, item_type: &ItemType) -> Option<&'a Self::TNext>;
}
