use crate::{attributes::affected_by::AffectedBy, traits::handles_effects::Effect};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Force;

impl Effect for Force {
	type TTarget = AffectedBy<Force>;
}
