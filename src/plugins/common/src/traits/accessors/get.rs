use bevy::ecs::system::SystemParam;

mod assets;

pub trait Get<TKey> {
	type TValue;

	fn get(&self, key: &TKey) -> Option<Self::TValue>;
}

pub trait GetRef<TKey> {
	type TValue<'a>
	where
		Self: 'a;

	fn get_ref(&self, key: &TKey) -> Option<Self::TValue<'_>>;
}

pub type AsParamEntry<'w, 's, T, TKey> = <T as GetParamEntry<'w, 's, TKey>>::TEntry;
pub type AsParam<'w, 's, T, TKey> = <T as GetParamEntry<'w, 's, TKey>>::TParam;
pub type AsParamItem<'w, 's, 'w2, 's2, T, TKey> =
	<AsParam<'w, 's, T, TKey> as SystemParam>::Item<'w2, 's2>;

pub trait GetParamEntry<'w, 's, TKey> {
	type TParam: SystemParam;
	type TEntry;

	fn get_param_entry(
		&self,
		key: &TKey,
		assets: &<Self::TParam as SystemParam>::Item<'_, '_>,
	) -> Self::TEntry;
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

/// A getter style conversion ("into" from a reference)
///
/// A blanket implementation exists for types that implement `From<&SomeType>`, which - similar
/// to `Into` vs. `From` - should be preferably implemented.
pub trait RefInto<'a, TValue> {
	fn ref_into(&'a self) -> TValue;
}

impl<'a, TFrom, TInto> RefInto<'a, TInto> for TFrom
where
	TFrom: 'a,
	TInto: From<&'a TFrom> + 'a,
{
	fn ref_into(&'a self) -> TInto {
		TInto::from(self)
	}
}

/// Getter like blanket trait for calling [`RefInto::ref_into()`]
///
/// Allows more explicit type conversion, like:
/// ```
/// use common::traits::accessors::get::RefAs;
///
/// struct MySpaceShip;
///
/// #[derive(Debug, PartialEq)]
/// enum FtlMethod {
///   Warp,
///   Wormhole,
/// }
///
/// impl From<&MySpaceShip> for FtlMethod {
///   fn from(_: &MySpaceShip) -> FtlMethod {
///     FtlMethod::Wormhole
///   }
/// }
///
/// let ship = MySpaceShip;
///
/// assert_eq!(FtlMethod::Wormhole, ship.ref_as::<FtlMethod>());
/// ```
pub trait RefAs {
	fn ref_as<'a, T>(&'a self) -> T
	where
		Self: RefInto<'a, T>;
}

impl<TSource> RefAs for TSource {
	fn ref_as<'a, T>(&'a self) -> T
	where
		Self: RefInto<'a, T>,
	{
		self.ref_into()
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
