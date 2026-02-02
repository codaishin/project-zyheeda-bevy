use crate::traits::{
	handles_physics::physical_bodies::Blockers,
	handles_skill_physics::SkillShape,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Projectile {
	pub destroyed_by: Blockers,
}

impl From<Projectile> for SkillShape {
	fn from(projectile: Projectile) -> Self {
		Self::Projectile(projectile)
	}
}
