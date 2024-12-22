use super::UnitsPerSecond;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Speed(pub UnitsPerSecond);

impl From<UnitsPerSecond> for Speed {
	fn from(speed: UnitsPerSecond) -> Self {
		Speed(speed)
	}
}
