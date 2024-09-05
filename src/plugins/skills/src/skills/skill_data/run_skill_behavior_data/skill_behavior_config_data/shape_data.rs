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

impl<TLifeTimeIn, TLifeTimeOut> From<SkillShapeData<TLifeTimeIn>> for BuildSkillShape<TLifeTimeOut>
where
	TLifeTimeOut: From<TLifeTimeIn>,
{
	fn from(value: SkillShapeData<TLifeTimeIn>) -> Self {
		match value {
			SkillShapeData::GroundTargetedAoe(v) => Self::GroundTargetedAoe(v.map_lifetime()),
			SkillShapeData::Projectile(v) => Self::Projectile(v),
			SkillShapeData::Shield(v) => Self::Shield(v),
		}
	}
}
