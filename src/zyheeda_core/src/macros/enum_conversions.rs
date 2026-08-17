/// Implement conversions of enums to and from the inner types of the listed
/// enum variants.
///
/// It also allows direct comparison of inner and outer type instances.
///
/// # Example
/// ```
/// use zyheeda_core::prelude::*;
///
/// #[derive(Debug, PartialEq)]
/// enum Animal {
///   Cat(Cat),
///   Dog(Dog),
/// }
///
/// #[derive(Debug, PartialEq)]
/// struct Dog;
///
/// #[derive(Debug, PartialEq)]
/// struct Cat;
///
/// impl_enum_conversions!(Animal[
///   Dog(Dog),
///   Cat(Cat),
/// ]);
///
/// assert_eq!(Animal::Dog(Dog), Animal::from(Dog));
///
/// assert_eq!(Ok(Cat), Cat::try_from(Animal::Cat(Cat)));
/// assert_eq!(Err(IsNot::target_type()), Cat::try_from(Animal::Dog(Dog)));
///
/// assert_eq!(Cat, Animal::Cat(Cat));
/// assert_ne!(Dog, Animal::Cat(Cat));
///
/// assert_eq!(Animal::Cat(Cat), Cat);
/// assert_ne!(Animal::Cat(Cat), Dog);
/// ```
#[macro_export]
macro_rules! impl_enum_conversions {
	($outer:ty[$($variant:ident($inner:ty)),+ $(,)?]) => {
		$(
			impl From<$inner> for $outer {
				fn from(inner: $inner) -> Self {
					Self::$variant(inner)
				}
			}

			impl TryFrom<$outer> for $inner {
				type Error = $crate::prelude::IsNot<$inner>;

				fn try_from(outer: $outer) -> Result<$inner, Self::Error> {
					type Outer = $outer;

					match outer {
						Outer::$variant(inner) => Ok(inner),
						_ => Err($crate::prelude::IsNot::target_type()),
					}
				}
			}

			impl PartialEq<$outer> for $inner
			where
				$inner: PartialEq  // Provides clear error message when constraint not implemented
			{
				fn eq(&self, other: &$outer) -> bool {
					type Outer = $outer;

					match other {
						Outer::$variant(other) => $crate::macros::enum_conversions::eq(self, other),
						_ => false,
					}
				}
			}

			impl PartialEq<$inner> for $outer
			where
				$inner: PartialEq  // Provides clear error message when constraint not implemented
			{
				fn eq(&self, other: &$inner) -> bool {
					type Outer = $outer;

					match self {
						Outer::$variant(this) => $crate::macros::enum_conversions::eq(this, other),
						_ => false,
					}
				}
			}
		)+
	};
}

pub fn eq<T: PartialEq>(left: &T, right: &T) -> bool {
	left == right
}

pub use impl_enum_conversions;

#[cfg(test)]
mod tests {
	use crate::conversion::is_not::IsNot;
	use std::{fmt::Debug, marker::PhantomData};
	use test_case::test_case;

	#[derive(Debug, PartialEq)]
	enum _Outer {
		A(_InnerA),
		B(_InnerB),
	}

	#[derive(Debug, PartialEq)]
	struct _InnerA;

	#[derive(Debug, PartialEq)]
	struct _InnerB;

	impl_enum_conversions!(_Outer[
		A(_InnerA),
		B(_InnerB),
	]);

	#[test_case(_InnerA, _Outer::A(_InnerA); "a")]
	#[test_case(_InnerB, _Outer::B(_InnerB); "b")]
	fn to_outer<TInner>(inner: TInner, outer: _Outer)
	where
		_Outer: From<TInner>,
	{
		assert_eq!(outer, _Outer::from(inner));
	}

	#[test_case(_Outer::A(_InnerA), _InnerA; "a")]
	#[test_case(_Outer::B(_InnerB), _InnerB; "b")]
	fn try_to_inner<TInner>(outer: _Outer, inner: TInner)
	where
		TInner: TryFrom<_Outer, Error = IsNot<TInner>> + Debug + PartialEq,
	{
		assert_eq!(Ok(inner), TInner::try_from(outer));
	}

	#[test_case(_Outer::A(_InnerA), PhantomData::<_InnerB>; "a")]
	#[test_case(_Outer::B(_InnerB), PhantomData::<_InnerA>; "b")]
	fn try_to_inner_error<TInner>(outer: _Outer, _: PhantomData<TInner>)
	where
		TInner: TryFrom<_Outer, Error = IsNot<TInner>> + Debug + PartialEq,
	{
		assert_eq!(Err(IsNot::target_type()), TInner::try_from(outer));
	}

	#[test_case(_Outer::A(_InnerA), _InnerA; "a")]
	#[test_case(_Outer::B(_InnerB), _InnerB; "b")]
	fn equal_with_outer<TInner>(outer: _Outer, inner: TInner)
	where
		TInner: Debug + PartialEq<_Outer>,
	{
		assert_eq!(inner, outer);
	}

	#[test_case(_Outer::A(_InnerA), _InnerB; "a")]
	#[test_case(_Outer::B(_InnerB), _InnerA; "b")]
	fn unequal_with_outer<TInner>(outer: _Outer, inner: TInner)
	where
		TInner: Debug + PartialEq<_Outer>,
	{
		assert_ne!(inner, outer);
	}

	#[test_case(_Outer::A(_InnerA), _InnerA; "a")]
	#[test_case(_Outer::B(_InnerB), _InnerB; "b")]
	fn equal_with_inner<TInner>(outer: _Outer, inner: TInner)
	where
		TInner: Debug,
		_Outer: PartialEq<TInner>,
	{
		assert_eq!(outer, inner);
	}

	#[test_case(_Outer::A(_InnerA), _InnerB; "a")]
	#[test_case(_Outer::B(_InnerB), _InnerA; "b")]
	fn unequal_with_inner<TInner>(outer: _Outer, inner: TInner)
	where
		TInner: Debug,
		_Outer: PartialEq<TInner>,
	{
		assert_ne!(outer, inner);
	}
}
