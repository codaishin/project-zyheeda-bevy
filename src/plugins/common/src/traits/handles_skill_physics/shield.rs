use crate::traits::handles_skill_physics::SkillShape;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Shield;

impl From<Shield> for SkillShape {
	fn from(shield: Shield) -> Self {
		Self::Shield(shield)
	}
}
