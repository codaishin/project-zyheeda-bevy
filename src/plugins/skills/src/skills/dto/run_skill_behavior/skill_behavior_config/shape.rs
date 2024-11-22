use crate::behaviors::build_skill_shape::{
	spawn_ground_target::SpawnGroundTargetedAoe,
	spawn_projectile::SpawnProjectile,
	spawn_shield::SpawnShield,
	BuildSkillShape,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum SkillShapeDto<T> {
	GroundTargetedAoe(SpawnGroundTargetedAoe<T>),
	Projectile(SpawnProjectile),
	Shield(SpawnShield),
}

impl<TLifeTimeIn, TLifeTimeOut> From<SkillShapeDto<TLifeTimeIn>> for BuildSkillShape<TLifeTimeOut>
where
	TLifeTimeOut: From<TLifeTimeIn>,
{
	fn from(value: SkillShapeDto<TLifeTimeIn>) -> Self {
		match value {
			SkillShapeDto::GroundTargetedAoe(v) => Self::GroundTargetedAoe(v.map_lifetime()),
			SkillShapeDto::Projectile(v) => Self::Projectile(v),
			SkillShapeDto::Shield(v) => Self::Shield(v),
		}
	}
}
