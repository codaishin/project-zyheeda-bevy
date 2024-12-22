use super::Units;
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Copy, Default, PartialOrd)]
pub struct AggroRange(pub Units);

impl From<Units> for AggroRange {
	fn from(range: Units) -> Self {
		AggroRange(range)
	}
}

impl Deref for AggroRange {
	type Target = Units;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
