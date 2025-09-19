use crate::traits::accessors::get::Property;
use bevy::prelude::*;

impl Property for Ray3d {
	type TValue<'a> = Self;
}
