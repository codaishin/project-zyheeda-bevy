use super::Units;
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Copy, Default, PartialOrd)]
pub struct AttackRange(pub Units);

impl From<Units> for AttackRange {
	fn from(range: Units) -> Self {
		AttackRange(range)
	}
}

impl Deref for AttackRange {
	type Target = Units;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
