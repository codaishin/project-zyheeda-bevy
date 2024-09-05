use crate::behaviors::build_skill_shape::{
	spawn_ground_target::SpawnGroundTargetedAoe,
	spawn_projectile::SpawnProjectile,
	spawn_shield::SpawnShield,
	BuildSkillShape,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum SkillShapeData<T> {
	GroundTargetedAoe(SpawnGroundTargetedAoe<T>),
	Projectile(SpawnProjectile),
	Shield(SpawnShield),
}

impl<TLifeTime> From<SkillShapeData<TLifeTime>> for BuildSkillShape<TLifeTime> {
	fn from(value: SkillShapeData<TLifeTime>) -> Self {
		match value {
			SkillShapeData::GroundTargetedAoe(v) => Self::GroundTargetedAoe(v),
			SkillShapeData::Projectile(v) => Self::Projectile(v),
			SkillShapeData::Shield(v) => Self::Shield(v),
		}
	}
}
