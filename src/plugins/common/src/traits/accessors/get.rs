mod assets;
mod components;
mod entity;
mod handle;
mod option;
mod ray;
mod result;

use bevy::ecs::system::{SystemParam, SystemParamItem};

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

pub trait ContextChanged {
	fn context_changed(&self) -> bool;
}

/// Retrieve a context for data inspection.
///
/// It is up to the implementor, what kind of system parameters are involved.
pub trait GetContext<TKey>: SystemParam {
	type TContext<'ctx>: ContextChanged;

	fn get_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		key: TKey,
	) -> Option<Self::TContext<'ctx>>;
}

pub trait GetChangedContext<TKey>: GetContext<TKey> {
	fn get_changed_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		key: TKey,
	) -> Option<Self::TContext<'ctx>>;
}

impl<T, TKey> GetChangedContext<TKey> for T
where
	T: GetContext<TKey>,
{
	fn get_changed_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		key: TKey,
	) -> Option<Self::TContext<'ctx>> {
		match T::get_context(param, key) {
			Some(ctx) if ctx.context_changed() => Some(ctx),
			_ => None,
		}
	}
}

pub trait GetMut<TKey> {
	type TValue<'a>
	where
		Self: 'a;

	fn get_mut(&mut self, key: &TKey) -> Option<Self::TValue<'_>>;
}

/// Retrieve a context for data mutation.
///
/// It is up to the implementor, what kind of system parameters are involved.
pub trait GetContextMut<TKey>: SystemParam {
	type TContext<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SystemParamItem<Self>,
		key: TKey,
	) -> Option<Self::TContext<'ctx>>;
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

/// A type-level descriptor for use with [`GetProperty`].
///
/// Implementors of this trait define how a value should be accessed through [`GetProperty`].
/// The associated type [`TValue<'a>`](Property::TValue) determines whether the value is retrieved
/// by reference (useful for computation-heavy or non-`Clone` types) or by value.
///
/// This can also be used for wrappers or markers to expose a different type.
/// # Example
/// ```
/// use common::traits::accessors::get::{Property};
///
/// // Small types can be returned by value.
/// #[derive(Clone, Copy)]
/// enum TinyEnum {
///   A,
///   B,
/// }
///
/// impl Property for TinyEnum {
///   type TValue<'a> = Self;
/// }
///
/// // Large or expensive-to-clone types are better returned by reference.
/// struct GiantArray([String; 100_000]);
///
/// impl Property for GiantArray {
///   type TValue<'a> = &'a Self;
/// }
///
/// // Wrapper types can choose to expose their inner value directly,
/// // independent of how the value itself is stored.
/// struct NumberOfLimbs(u8);
///
/// impl Property for NumberOfLimbs {
///   type TValue<'a> = u8;
/// }
///
/// // Markers, that don't hold data, can be used to name a property
/// struct NumberOfDoors;
///
/// impl Property for NumberOfDoors {
///   type TValue<'a> = u8;
/// }
/// ```
pub trait Property {
	type TValue<'a>;
}

/// A generic getter trait over a [`Property`].
///
/// The property specifies how the value is exposed (by reference, by value, or via a transformed
/// inner type). This allows consumers to request a value in a uniform way, while letting the
/// property control the retrieval strategy.
///
/// # Example
/// ```
/// use common::traits::accessors::get::{Property, GetProperty};
///
/// #[derive(Debug, PartialEq)]
/// struct MyProperty;
///
/// impl Property for MyProperty {
///   type TValue<'a> = Self;
/// }
///
/// struct Obj;
///
/// impl GetProperty<MyProperty> for Obj {
///   fn get_property(&self) -> MyProperty {
///     MyProperty
///   }
/// }
///
/// let obj = Obj;
///
/// assert_eq!(MyProperty, obj.get_property());
/// ```
pub trait GetProperty<TProperty>
where
	TProperty: Property,
{
	fn get_property(&self) -> TProperty::TValue<'_>;
}

/// A convenience trait to access any [`Property`] dynamically.
///
/// Useful when you want to name the retrieved property directly on the call.
///
/// ```
/// use common::traits::accessors::get::{Property, GetProperty, DynProperty};
///
/// #[derive(Debug, PartialEq)]
/// struct MyProperty;
///
/// impl Property for MyProperty {
///   type TValue<'a> = Self;
/// }
///
/// struct Obj;
///
/// impl GetProperty<MyProperty> for Obj {
///   fn get_property(&self) -> MyProperty {
///     MyProperty
///   }
/// }
///
/// let obj = Obj;
///
/// // when rust cannot determine which `GetProperty` impl of many to use:
/// let a: MyProperty = obj.get_property();
///
/// // same issue as above, but using `DynProperty`:
/// let b = obj.dyn_property::<MyProperty>();
///
/// assert_eq!(a, b);
/// ```
pub trait DynProperty {
	fn dyn_property<TProperty>(&self) -> TProperty::TValue<'_>
	where
		TProperty: Property,
		Self: GetProperty<TProperty>;
}

impl<T> DynProperty for T {
	fn dyn_property<TProperty>(&self) -> TProperty::TValue<'_>
	where
		TProperty: Property,
		Self: GetProperty<TProperty>,
	{
		GetProperty::<TProperty>::get_property(self)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::{collections::HashMap, fmt::Debug};

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
