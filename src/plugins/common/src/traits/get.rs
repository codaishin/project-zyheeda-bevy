pub trait Get<TKey, TValue> {
	fn get(&self, key: &TKey) -> Option<&TValue>;
}

pub trait GetStatic<TValue> {
	fn get(&self) -> &TValue;
}
