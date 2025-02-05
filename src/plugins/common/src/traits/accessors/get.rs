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

pub trait GetterRefOptional<TValue> {
	fn get(&self) -> Option<&TValue>;
}

pub trait Getter<TValue> {
	fn get(&self) -> TValue;
}

pub trait GetField<TWithGetter> {
	fn get_field(source: &TWithGetter) -> Self;
}

impl<T, TWithGetter> GetField<TWithGetter> for T
where
	TWithGetter: Getter<T>,
{
	fn get_field(source: &TWithGetter) -> Self {
		source.get()
	}
}

pub trait GetFieldRef<TWithGetterRef> {
	fn get_field_ref(source: &TWithGetterRef) -> &Self;
}

impl<T, TWithGetterRef> GetFieldRef<TWithGetterRef> for T
where
	TWithGetterRef: GetterRef<T>,
{
	fn get_field_ref(source: &TWithGetterRef) -> &Self {
		source.get()
	}
}
