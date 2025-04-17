use super::UnitsPerSecond;
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
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
