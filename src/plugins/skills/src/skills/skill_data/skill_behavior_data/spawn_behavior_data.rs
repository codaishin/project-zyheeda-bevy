use crate::behaviors::spawn_behavior::{
	spawn_ground_target::SpawnGroundTargetedAoe,
	spawn_projectile::SpawnProjectile,
	spawn_shield::SpawnShield,
	SkillShape,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum SkillShapeData {
	Placeholder, // Fixme: this is temporary, until we establish proper modularization of all skills
	GroundTargetedAoe(SpawnGroundTargetedAoe),
	Projectile(SpawnProjectile),
	Shield(SpawnShield),
}

impl From<SkillShapeData> for SkillShape {
	fn from(value: SkillShapeData) -> Self {
		match value {
			SkillShapeData::Placeholder => Self::default(),
			SkillShapeData::GroundTargetedAoe(v) => Self::GroundTargetedAoe(v),
			SkillShapeData::Projectile(v) => Self::Projectile(v),
			SkillShapeData::Shield(v) => Self::Shield(v),
		}
	}
}
