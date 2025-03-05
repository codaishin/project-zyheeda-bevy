use super::Units;
use std::ops::Deref;

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
