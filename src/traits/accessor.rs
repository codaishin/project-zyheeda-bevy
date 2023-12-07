pub mod tuple_slot_key_item;

pub trait Accessor<TContainer, TGet, TSet> {
	fn get_key_and_item(&self, container: &TContainer) -> Option<TGet>;
	fn with_item(&self, item: Option<TSet>, container: &mut TContainer) -> Self;
}
