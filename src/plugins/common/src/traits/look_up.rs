pub trait LookUp<TKey, TValue> {
	fn get(&self, key: &TKey) -> Option<&TValue>;
}
