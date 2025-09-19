use crate::traits::accessors::get::Property;
use bevy::asset::{Asset, Handle};

impl<T> Property for Handle<T>
where
	T: Asset,
{
	type TValue<'a> = &'a Self;
}
