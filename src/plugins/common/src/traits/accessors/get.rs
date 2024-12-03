mod assets;

pub trait Get<TKey, TValue> {
	fn get(&self, key: &TKey) -> Option<TValue>;
}

pub trait GetRef<TKey, TValue> {
	fn get(&self, key: &TKey) -> Option<&TValue>;
}

pub trait GetMut<TKey, TValue> {
	fn get_mut(&mut self, key: &TKey) -> Option<&mut TValue>;
}

pub trait GetterRef<TValue> {
	fn get(&self) -> &TValue;
}

pub trait Getter<TValue> {
	fn get(&self) -> TValue;
}
