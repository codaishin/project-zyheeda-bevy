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

/// A type-level descriptor for use with [`View`] or [`ViewOf`].
///
/// Implementers of this trait define how a value should be accessed through [`View`] or
/// [`ViewOf`]. The associated type [`TValue<'a>`](ViewField::TValue) determines whether the
/// value is retrieved by reference (useful for computation-heavy or non-`Clone` types) or by value.
///
/// This can also be used for wrappers or markers to expose a different type.
///
/// # Example
/// ```
/// use common::traits::accessors::get::{ViewField};
///
/// // Small types can be returned by value.
/// #[derive(Clone, Copy)]
/// enum TinyEnum {
///   A,
///   B,
/// }
///
/// impl ViewField for TinyEnum {
///   type TValue<'a> = Self;
/// }
///
/// // Large or expensive-to-clone types are better returned by reference.
/// struct GiantArray([String; 100_000]);
///
/// impl ViewField for GiantArray {
///   type TValue<'a> = &'a Self;
/// }
///
/// // Wrapper types can choose to expose their inner value directly
/// struct NumberOfLimbs(u8);
///
/// impl ViewField for NumberOfLimbs {
///   type TValue<'a> = u8;
/// }
///
/// // Markers, that don't hold data, can be used to name a property
/// struct NumberOfDoors;
///
/// impl ViewField for NumberOfDoors {
///   type TValue<'a> = u8;
/// }
/// ```
pub trait ViewField {
	type TValue<'a>;
}

/// View a [`ViewField`].
///
/// The view field type specifies how the value is exposed (by reference, by value, or via a
/// transformed type). This allows consumers to request a value in a uniform way, while letting the
/// view field type control the retrieval strategy.
///
/// # Example
/// ```
/// use common::traits::accessors::get::{ViewField, View};
///
/// #[derive(Debug, PartialEq)]
/// struct MyField;
///
/// impl ViewField for MyField {
///   type TValue<'a> = Self;
/// }
///
/// struct MyObj;
///
/// impl View<MyField> for MyObj {
///   fn view(&self) -> MyField {
///     MyField
///   }
/// }
///
/// let obj = MyObj;
///
/// assert_eq!(MyField, obj.view());
/// ```
pub trait View<TField>
where
	TField: ViewField,
{
	fn view(&self) -> TField::TValue<'_>;
}

/// A convenience trait to access any [`ViewField`] by naming it.
///
/// Useful when you want to name the retrieved property directly on the call.
///
/// ```
/// use common::traits::accessors::get::{ViewField, View, ViewOf};
///
/// #[derive(Debug, PartialEq)]
/// struct MyField;
///
/// impl ViewField for MyField {
///   type TValue<'a> = Self;
/// }
///
/// struct MyObj;
///
/// impl View<MyField> for MyObj {
///   fn view(&self) -> MyField {
///     MyField
///   }
/// }
///
/// let obj = MyObj;
///
/// // naming the field type in case of ambiguity between multiple `View<T>` impls
/// let a: MyField = obj.view();
///
/// assert_eq!(MyField, a);
///
/// // naming the full `View` trait in case of ambiguity between multiple `View<T>` impls
/// let b = View::<MyField>::view(&obj);
///
/// assert_eq!(MyField, b);
///
/// // using `ViewOf` in case of ambiguity between multiple `View<T>` impls
/// let c = obj.view_of::<MyField>();
///
/// assert_eq!(MyField, c);
/// ```
pub trait ViewOf {
	fn view_of<TField>(&self) -> TField::TValue<'_>
	where
		TField: ViewField,
		Self: View<TField>;
}

impl<T> ViewOf for T {
	fn view_of<TField>(&self) -> TField::TValue<'_>
	where
		TField: ViewField,
		Self: View<TField>,
	{
		View::<TField>::view(self)
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
