use super::UnitsPerSecond;
use crate::traits::accessors::get::ViewField;
use macros::serde_model;
use std::ops::Deref;

#[serde_model]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Speed(pub UnitsPerSecond);

impl Speed {
	pub const ZERO: Self = Self(UnitsPerSecond::ZERO);
}

impl From<UnitsPerSecond> for Speed {
	fn from(speed: UnitsPerSecond) -> Self {
		Speed(speed)
	}
}

impl Deref for Speed {
	type Target = f32;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl ViewField for Speed {
	type TValue<'a> = UnitsPerSecond;
}
