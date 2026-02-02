use crate::{
	tools::Units,
	traits::{handles_physics::physical_bodies::Blockers, handles_skill_physics::SkillShape},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Beam {
	pub range: Units,
	pub blocked_by: Blockers,
}

impl From<Beam> for SkillShape {
	fn from(beam: Beam) -> Self {
		Self::Beam(beam)
	}
}
