use super::projectile_type::ProjectileType;
use crate::{skills::SkillBehavior, traits::GetStaticSkillBehavior};
use behaviors::components::{gravity_well::GravityWell, ForceShield};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum SkillBehaviorData {
	Projectile(ProjectileType),
	ForceShield,
	GravityWell,
}

impl From<SkillBehaviorData> for SkillBehavior {
	fn from(value: SkillBehaviorData) -> Self {
		match value {
			SkillBehaviorData::ForceShield => ForceShield::behavior(),
			SkillBehaviorData::GravityWell => GravityWell::behavior(),
			SkillBehaviorData::Projectile(ty) => ty.projectile_behavior(),
		}
	}
}
