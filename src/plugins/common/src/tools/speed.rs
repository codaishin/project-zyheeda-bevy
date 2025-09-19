use crate::traits::accessors::get::Property;

use super::UnitsPerSecond;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Speed(pub UnitsPerSecond);

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

impl Property for Speed {
	type TValue<'a> = UnitsPerSecond;
}
