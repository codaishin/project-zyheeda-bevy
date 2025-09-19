use crate::traits::accessors::get::Property;

#[derive(Debug, PartialEq)]
pub struct AttributeOnSpawn<T>(pub T);

impl<T> Property for AttributeOnSpawn<T> {
	type TValue<'a> = T;
}
