use crate::behaviors::build_skill_shape::{
	BuildSkillShape,
	spawn_ground_target::SpawnGroundTargetedAoe,
	spawn_projectile::SpawnProjectile,
	spawn_shield::SpawnShield,
};
use common::dto::duration::DurationDto;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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

impl From<BuildSkillShape> for SkillShapeDto {
	fn from(value: BuildSkillShape) -> Self {
		match value {
			BuildSkillShape::GroundTargetedAoe(v) => SkillShapeDto::GroundTargetedAoe(v.into()),
			BuildSkillShape::Projectile(v) => SkillShapeDto::Projectile(v),
			BuildSkillShape::Shield(v) => SkillShapeDto::Shield(v),
			#[cfg(test)]
			BuildSkillShape::Fn(_) => panic!("FN CANNOT BE SERIALIZED"),
		}
	}
}
