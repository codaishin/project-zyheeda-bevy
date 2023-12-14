pub mod inventory;
pub mod inventory_panel;
pub mod slots;

pub trait Get<TKey, TValue> {
	fn get(&self, key: TKey) -> TValue;
}
