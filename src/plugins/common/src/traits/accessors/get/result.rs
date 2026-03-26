use crate::traits::accessors::get::ViewField;

impl<T, TError> ViewField for Result<T, TError>
where
	T: ViewField,
{
	type TValue<'a> = Result<T::TValue<'a>, TError>;
}
