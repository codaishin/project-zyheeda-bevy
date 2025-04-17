use common::tools::{item_type::ItemType, keys::slot::SlotKey};

pub(crate) trait PeekNextRecursive {
	type TNext<'a>
	where
		Self: 'a;
	type TRecursiveNode<'a>
	where
		Self: 'a;

	fn peek_next_recursive<'a>(
		&'a self,
		trigger: &SlotKey,
		item_type: &ItemType,
	) -> Option<(Self::TNext<'a>, Self::TRecursiveNode<'a>)>;
}
