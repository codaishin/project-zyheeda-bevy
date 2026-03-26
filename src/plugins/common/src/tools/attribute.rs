use crate::traits::accessors::get::ViewField;

#[derive(Debug, PartialEq)]
pub struct AttributeOnSpawn<T>(pub T);

impl<T> ViewField for AttributeOnSpawn<T> {
	type TValue<'a> = T;
}
