mod assets;

pub trait GetOption<TKey, TValue> {
	fn get(&self, key: &TKey) -> Option<TValue>;
}

pub trait GetRefOption<TKey, TValue> {
	fn get(&self, key: &TKey) -> Option<&TValue>;
}

pub trait GetMutOption<TKey, TValue> {
	fn get_mut(&mut self, key: &TKey) -> Option<&mut TValue>;
}

pub trait GetRef<TKey, TValue> {
	fn get(&self, key: &TKey) -> &TValue;
}

pub trait GetterRef<TValue> {
	fn get(&self) -> &TValue;
}

pub trait GetterMut<TValue> {
	fn get_mut(&mut self) -> &mut TValue;
}

pub trait Getter<TValue> {
	fn get(&self) -> TValue;
}
