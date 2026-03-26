use crate::traits::accessors::get::ViewField;
use bevy::asset::{Asset, Handle};

impl<T> ViewField for Handle<T>
where
	T: Asset,
{
	type TValue<'a> = &'a Self;
}
