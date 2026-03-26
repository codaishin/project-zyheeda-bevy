use super::Units;
use crate::traits::accessors::get::ViewField;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Default, Clone, Copy, Serialize, Deserialize)]
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
