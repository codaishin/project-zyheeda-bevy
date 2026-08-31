#[cfg(test)]
mod tests_enum_conversion_derive {
	use crate::conversion::is_not::IsNot;
	use macros::EnumConversions;
	use std::{fmt::Debug, marker::PhantomData};

	mod no_generics {

		use super::*;
		use test_case::test_case;

		#[derive(EnumConversions, Debug, PartialEq)]
		enum _Outer {
			A(_InnerA),
			B(_InnerB),
		}

		#[derive(Debug, PartialEq)]
		struct _InnerA;

		#[derive(Debug, PartialEq)]
		struct _InnerB;

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

		#[derive(EnumConversions, Debug, PartialEq)]
		enum _Outer<T = ()> {
			A(_InnerA),
			B(_InnerB),
			#[enum_conversions(skip)]
			_Other(T),
		}

		#[derive(Debug, PartialEq)]
		struct _InnerA;

		#[derive(Debug, PartialEq)]
		struct _InnerB;

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
