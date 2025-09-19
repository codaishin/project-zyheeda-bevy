use crate::traits::accessors::get::Property;

impl<T, TError> Property for Result<T, TError>
where
	T: Property,
{
	type TValue<'a> = Result<T::TValue<'a>, TError>;
}
