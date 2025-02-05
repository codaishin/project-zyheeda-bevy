use common::tools::{item_type::ItemType, slot_key::SlotKey};

pub(crate) trait PeekNextRecursive {
	type TNext;
	type TRecursiveNode;

	fn peek_next_recursive(
		&self,
		trigger: &SlotKey,
		item_type: &ItemType,
	) -> Option<(Self::TNext, Self::TRecursiveNode)>;
}
