use crate::{
	attributes::affected_by::AffectedBy,
	tools::UnitsPerSecond,
	traits::handles_effects::Effect,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Gravity {
	pub strength: UnitsPerSecond,
}

impl Effect for Gravity {
	type TTarget = AffectedBy<Gravity>;
}
