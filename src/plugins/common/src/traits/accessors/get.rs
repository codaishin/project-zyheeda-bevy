mod assets;
mod entity;
mod handle;
mod option;
mod ray;
mod result;

use bevy::ecs::system::{StaticSystemParam, SystemParam};
use std::ops::Deref;

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

pub type AssociatedStaticSystemParam<'world_self, 'state_self, 'world, 'state, T, TKey> =
	StaticSystemParam<
		'world_self,
		'state_self,
		<T as GetFromSystemParam<TKey>>::TParam<'world, 'state>,
	>;
pub type AssociatedSystemParam<'world_self, 'state_self, 'world, 'state, T, TKey> =
	<AssociatedStaticSystemParam<'world_self, 'state_self, 'world, 'state, T, TKey> as Deref>::Target;

/// Allows to retrieve data from a source, which only holds a part of or a reference to some data.
///
/// We can inject the required parameter in a system as a generic type to retrieve the full data.
/// Helper types exist due to the involved life time complexities and the required
/// [`StaticSystemParam`] for generic system parameters.
///
/// Note that the below example would also work without using [`StaticSystemParam`], due to rust
/// being able to immediately name the actual type of the required system parameter. However, this
/// does not work for injecting types across generic plugin borders.
///
/// # Example
/// ```
/// use common::traits::accessors::get::{AssociatedStaticSystemParam, GetFromSystemParam};
/// use bevy::{ecs::system::RunSystemOnce, prelude::*};
/// use std::fmt::Display;
///
/// #[derive(Resource)]
/// struct Displays {
///   short: Vec<String>,
///   long: Vec<String>,
/// }
///
/// enum Length {
///   Short,
///   Long,
/// }
///
/// #[derive(Component)]
/// struct Index(usize);
///
/// impl GetFromSystemParam<Length> for Index {
///   type TParam<'w, 's> = Res<'w, Displays>;
///   type TItem<'i> = &'i str;
///
///   fn get_from_param<'a>(
///     &self,
///     length: &Length,
///     displays: &'a Res<Displays>,
///   ) -> Option<&'a str> {
///     let displays = match length {
///       Length::Short => &displays.short,
///       Length::Long => &displays.long,
///     };
///
///     displays.get(self.0).map(String::as_str)
///   }
/// }
///
/// fn my_system<TSource>(q: Query<&TSource>, p: AssociatedStaticSystemParam<TSource, Length>)
/// where
///   TSource: Component + GetFromSystemParam<Length>,
///   for <'i> TSource::TItem<'i>: Display,
/// {
///   for source in &q {
///     let display = source.get_from_param(&Length::Short, &p).unwrap();
///     assert_eq!("my short display", display.to_string());
///   }
/// }
///
/// let mut app = App::new();
/// app.insert_resource(Displays {
///   short: vec!["my short display".to_string()],
///   long: vec!["my long display".to_string()],
/// });
/// app.world_mut().spawn(Index(0));
///
/// assert!(app.world_mut().run_system_once(my_system::<Index>).is_ok());
/// ```
pub trait GetFromSystemParam<TKey> {
	type TParam<'world, 'state>: SystemParam;
	type TItem<'item>
	where
		Self: 'item;

	fn get_from_param<'a>(
		&'a self,
		key: &TKey,
		param: &'a AssociatedSystemParam<'_, '_, '_, '_, Self, TKey>,
	) -> Option<Self::TItem<'a>>;
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
