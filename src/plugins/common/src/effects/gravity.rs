use crate::tools::UnitsPerSecond;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Gravity {
	pub strength: UnitsPerSecond,
}
