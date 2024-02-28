pub trait Accessor<TContainer, TGet, TSet> {
	fn get_key_and_item(&self, container: &TContainer) -> TGet;
	fn with_item(&self, item: Option<TSet>, container: &mut TContainer) -> Self;
}
