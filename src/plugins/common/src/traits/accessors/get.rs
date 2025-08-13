mod assets;

pub trait Get<TKey> {
	type TValue;

	fn get(&self, key: &TKey) -> Option<Self::TValue>;
}

pub trait GetRef<TKey> {
	type TValue<'a>
	where
		Self: 'a;

	fn get(&self, key: &TKey) -> Option<Self::TValue<'_>>;
}

pub trait GetMut<TKey> {
	type TValue<'a>
	where
		Self: 'a;

	fn get_mut(&mut self, key: &TKey) -> Option<Self::TValue<'_>>;
}

pub trait TryApplyOn<'a, TKey>: GetMut<TKey> + 'a {
	fn try_apply_on<TFn, TReturn>(&'a mut self, key: &TKey, apply: TFn) -> Option<TReturn>
	where
		TFn: FnOnce(Self::TValue<'a>) -> TReturn,
	{
		let value = self.get_mut(key)?;
		Some(apply(value))
	}
}

impl<'a, T, TEntity> TryApplyOn<'a, TEntity> for T where T: GetMut<TEntity> + 'a {}

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

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashMap;

	#[derive(Debug, PartialEq)]
	struct _Container(HashMap<&'static str, i32>);

	impl GetMut<&'static str> for _Container {
		type TValue<'a>
			= &'a mut i32
		where
			Self: 'a;

		fn get_mut(&mut self, key: &&'static str) -> Option<Self::TValue<'_>> {
			self.0.get_mut(key)
		}
	}

	#[test]
	fn try_apply_on() {
		let mut container = _Container(HashMap::from([("Foo", 42)]));

		container.try_apply_on(&"Foo", |v| *v = 11);

		assert_eq!(_Container(HashMap::from([("Foo", 11)])), container);
	}
}
