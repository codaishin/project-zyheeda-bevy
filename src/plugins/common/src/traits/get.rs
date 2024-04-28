pub trait Get<TKey, TValue> {
	fn get(&self, key: &TKey) -> &TValue;
}

pub trait GetStatic<TValue> {
	fn get(&self) -> &TValue;
}
