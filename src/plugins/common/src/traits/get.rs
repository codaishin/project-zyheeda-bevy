pub trait Get<TKey, TValue> {
	fn get(&self, key: &TKey) -> Option<&TValue>;
}

pub trait GetMut<TKey, TValue> {
	fn get_mut(&mut self, key: &TKey) -> Option<&mut TValue>;
}

pub trait GetStatic<TValue> {
	fn get(&self) -> &TValue;
}
