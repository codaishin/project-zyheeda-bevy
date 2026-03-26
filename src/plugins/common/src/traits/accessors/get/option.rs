use crate::traits::accessors::get::ViewField;

impl<T> ViewField for Option<T>
where
	T: ViewField,
{
	type TValue<'a> = Option<T::TValue<'a>>;
}
