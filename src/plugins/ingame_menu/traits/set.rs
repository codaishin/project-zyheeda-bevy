pub mod inventory_panel;

pub trait Set<TKey, TValue> {
	fn set(&mut self, key: TKey, value: TValue);
}
