pub mod item;
pub mod panel_state;

pub trait Get<TKey, TValue> {
	fn get(&self, key: TKey) -> TValue;
}
