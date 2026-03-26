use crate::traits::accessors::get::ViewField;
use bevy::prelude::*;

impl ViewField for Ray3d {
	type TValue<'a> = Self;
}
