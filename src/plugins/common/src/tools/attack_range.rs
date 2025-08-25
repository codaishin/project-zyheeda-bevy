use super::Units;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Copy, Default, PartialOrd, Serialize, Deserialize)]
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
