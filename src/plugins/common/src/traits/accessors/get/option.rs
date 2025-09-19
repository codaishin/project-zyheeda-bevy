use crate::traits::accessors::get::Property;

impl<T> Property for Option<T>
where
	T: Property,
{
	type TValue<'a> = Option<T::TValue<'a>>;
}
