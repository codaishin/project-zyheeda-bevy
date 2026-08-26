use super::Units;
use crate::traits::accessors::get::ViewField;
use macros::serde_model;
use std::ops::Deref;

#[serde_model]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct ColliderRadius(pub Units);

impl From<Units> for ColliderRadius {
	fn from(radius: Units) -> Self {
		Self(radius)
	}
}

impl Deref for ColliderRadius {
	type Target = Units;

	fn deref(&self) -> &Self::Target {
		let ColliderRadius(radius) = self;
		radius
	}
}

impl ViewField for ColliderRadius {
	type TValue<'a> = Units;
}
