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
		)+
	};
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
		assert_eq!(Err(IsNot::<TInner>::target_type()), TInner::try_from(outer));
	}
}
