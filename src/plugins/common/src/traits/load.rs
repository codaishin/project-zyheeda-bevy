pub mod asset_server;

pub trait Load<TKey, TValue> {
	fn load(&self, key: &TKey) -> TValue;
}
