use crate::behaviors::build_skill_shape::{
	BuildSkillShape,
	spawn_ground_target::SpawnGroundTargetedAoe,
	spawn_projectile::SpawnProjectile,
	spawn_shield::SpawnShield,
};
use common::dto::duration::DurationDto;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum SkillShapeDto {
	GroundTargetedAoe(SpawnGroundTargetedAoe<DurationDto>),
	Projectile(SpawnProjectile),
	Shield(SpawnShield),
}

impl From<SkillShapeDto> for BuildSkillShape {
	fn from(value: SkillShapeDto) -> Self {
		match value {
			SkillShapeDto::GroundTargetedAoe(v) => Self::GroundTargetedAoe(v.into()),
			SkillShapeDto::Projectile(v) => Self::Projectile(v),
			SkillShapeDto::Shield(v) => Self::Shield(v),
		}
	}
}
