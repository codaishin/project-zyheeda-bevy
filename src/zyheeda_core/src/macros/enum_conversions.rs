/// Implement conversions of enums to and from the inner types of the listed
/// enum variants.
///
/// It also allows direct comparison of inner and outer type instances.
///
/// # Example
/// ```
/// use zyheeda_core::prelude::*;
/// use std::marker::PhantomData;
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
/// impl_enum_conversions!(Animal::Dog(Dog));
/// impl_enum_conversions!(Animal::Cat(Cat));
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
///
/// // With generics
///
/// #[derive(Debug, PartialEq)]
/// enum Meta {}
///
/// #[derive(Debug, PartialEq)]
/// struct _Flying;
///
/// #[derive(Debug, PartialEq)]
/// enum MutatedAnimal<Mutation> {
///   Cat(Cat),
///   Dog(Dog),
///   _Mutation(PhantomData<Mutation>, Meta),
/// }
///
/// impl_enum_conversions!(<Mutation>: MutatedAnimal::<Mutation>::Dog(Dog));
/// impl_enum_conversions!(<Mutation>: MutatedAnimal::<Mutation>::Cat(Cat));
///
/// assert_eq!(MutatedAnimal::<_Flying>::Dog(Dog), MutatedAnimal::<_Flying>::from(Dog));
///
/// assert_eq!(Ok(Cat), Cat::try_from(MutatedAnimal::<_Flying>::Cat(Cat)));
/// assert_eq!(Err(IsNot::target_type()), Cat::try_from(MutatedAnimal::<_Flying>::Dog(Dog)));
///
/// assert_eq!(Cat, MutatedAnimal::<_Flying>::Cat(Cat));
/// assert_ne!(Dog, MutatedAnimal::<_Flying>::Cat(Cat));
///
/// assert_eq!(MutatedAnimal::<_Flying>::Cat(Cat), Cat);
/// assert_ne!(MutatedAnimal::<_Flying>::Cat(Cat), Dog);
///
/// // With conversions limited to specific generics
///
/// #[derive(Debug, PartialEq)]
/// enum MutatedAnimal2<Mutation> {
///   Cat(Cat),
///   Dog(Dog),
///   _Mutation(PhantomData<Mutation>, Meta),
/// }
///
/// impl_enum_conversions!(MutatedAnimal2::<_Flying>::Dog(Dog));
/// impl_enum_conversions!(MutatedAnimal2::<_Flying>::Cat(Cat));
///
/// assert_eq!(MutatedAnimal2::<_Flying>::Dog(Dog), MutatedAnimal2::<_Flying>::from(Dog));
///
/// assert_eq!(Ok(Cat), Cat::try_from(MutatedAnimal2::<_Flying>::Cat(Cat)));
/// assert_eq!(Err(IsNot::target_type()), Cat::try_from(MutatedAnimal2::<_Flying>::Dog(Dog)));
///
/// assert_eq!(Cat, MutatedAnimal2::<_Flying>::Cat(Cat));
/// assert_ne!(Dog, MutatedAnimal2::<_Flying>::Cat(Cat));
///
/// assert_eq!(MutatedAnimal2::<_Flying>::Cat(Cat), Cat);
/// assert_ne!(MutatedAnimal2::<_Flying>::Cat(Cat), Dog);
/// ```
#[macro_export]
macro_rules! impl_enum_conversions {
	($(<$($g_impl:ident),+>:)? $outer:ident$(::<$($g:ident),+>)?::$variant:ident($inner:ty)) => {
		impl $(<$($g_impl),+>)? From<$inner> for $outer$(<$($g),+>)? {
			fn from(inner: $inner) -> Self {
				Self::$variant(inner)
			}
		}

		impl $(<$($g_impl),+>)? TryFrom<$outer$(<$($g),+>)?> for $inner {
			type Error = $crate::prelude::IsNot<$inner>;

			fn try_from(outer: $outer$(<$($g),+>)?) -> Result<$inner, Self::Error> {

				match outer {
					$outer$(::<$($g),+>)?::$variant(inner) => Ok(inner),
					_ => Err($crate::prelude::IsNot::target_type()),
				}
			}
		}

		impl $(<$($g_impl),+>)? PartialEq<$outer$(<$($g),+>)?> for $inner
		where
			// Provide clear error message when constraint not implemented
			$inner: PartialEq,
			$($($g: PartialEq),+)?
		{
			fn eq(&self, other: &$outer$(<$($g),+>)?) -> bool {
				match other {
					$outer$(::<$($g),+>)?::$variant(other) => $crate::macros::enum_conversions::eq(self, other),
					_ => false,
				}
			}
		}

		impl $(<$($g_impl),+>)? PartialEq<$inner> for $outer$(<$($g),+>)?
		where
			// Provide clear error message when constraint not implemented
			$inner: PartialEq,
			$($($g: PartialEq),+)?
		{
			fn eq(&self, other: &$inner) -> bool {
				match self {
					$outer$(::<$($g),+>)?::$variant(this) => $crate::macros::enum_conversions::eq(this, other),
					_ => false,
				}
			}
		}
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

	mod no_generics {
		use super::*;
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

		impl_enum_conversions!(_Outer::A(_InnerA));
		impl_enum_conversions!(_Outer::B(_InnerB));

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

	mod generics {
		use super::*;
		use test_case::test_case;

		#[derive(Debug, PartialEq)]
		enum _DoNotBuild {}

		#[derive(Debug, PartialEq)]
		enum _Outer<T1 = (), T2 = ()> {
			A(_InnerA),
			B(_InnerB),
			_P(PhantomData<(T1, T2)>, _DoNotBuild),
		}

		#[derive(Debug, PartialEq)]
		struct _InnerA;

		#[derive(Debug, PartialEq)]
		struct _InnerB;

		impl_enum_conversions!(<T1, T2>: _Outer::<T1, T2>::A(_InnerA));
		impl_enum_conversions!(<T1, T2>: _Outer::<T1, T2>::B(_InnerB));

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

	mod concrete_generics {
		use super::*;
		use test_case::test_case;

		#[derive(Debug, PartialEq)]
		enum _DoNotBuild {}

		#[derive(Debug, PartialEq)]
		struct _T;

		#[derive(Debug, PartialEq)]
		enum _Outer<T1 = _T, T2 = _T> {
			A(_InnerA),
			B(_InnerB),
			_P(PhantomData<(T1, T2)>, _DoNotBuild),
		}

		#[derive(Debug, PartialEq)]
		struct _InnerA;

		#[derive(Debug, PartialEq)]
		struct _InnerB;

		impl_enum_conversions!(_Outer::<_T, _T>::A(_InnerA));
		impl_enum_conversions!(_Outer::<_T, _T>::B(_InnerB));

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
}
