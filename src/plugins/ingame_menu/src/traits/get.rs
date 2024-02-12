pub mod item;
pub mod panel_state;
pub mod slot_key;

pub trait Get<TKey, TValue> {
	fn get(&self, key: TKey) -> TValue;
}
