use crate::traits::handles_skill_physics::SkillShape;
use macros::serde_model;

#[serde_model]
#[derive(Debug, PartialEq, Clone)]
pub struct Shield;

impl From<Shield> for SkillShape {
	fn from(shield: Shield) -> Self {
		Self::Shield(shield)
	}
}
