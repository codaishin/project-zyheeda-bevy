use crate::traits::{
	handles_physics::physical_bodies::Blockers,
	handles_skill_physics::SkillShape,
};
use macros::serde_model;

#[serde_model]
#[derive(Debug, PartialEq, Clone)]
pub struct Projectile {
	pub destroyed_by: Blockers,
}

impl From<Projectile> for SkillShape {
	fn from(projectile: Projectile) -> Self {
		Self::Projectile(projectile)
	}
}
