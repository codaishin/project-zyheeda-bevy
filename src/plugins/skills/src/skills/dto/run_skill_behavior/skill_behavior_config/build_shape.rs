use crate::behaviors::build_skill_shape::{
	BuildSkillShape,
	spawn_ground_target::SpawnGroundTargetedAoe,
	spawn_projectile::SpawnProjectile,
	spawn_shield::SpawnShield,
};
use common::dto::duration_secs_f32::DurationSecsF32;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) enum BuildSkillShapeDto {
	GroundTargetedAoe(SpawnGroundTargetedAoe<DurationSecsF32>),
	Projectile(SpawnProjectile),
	Shield(SpawnShield),
}

impl From<BuildSkillShapeDto> for BuildSkillShape {
	fn from(value: BuildSkillShapeDto) -> Self {
		match value {
			BuildSkillShapeDto::GroundTargetedAoe(v) => {
				Self::GroundTargetedAoe(SpawnGroundTargetedAoe::from(v))
			}
			BuildSkillShapeDto::Projectile(v) => Self::Projectile(v),
			BuildSkillShapeDto::Shield(v) => Self::Shield(v),
		}
	}
}

impl From<BuildSkillShape> for BuildSkillShapeDto {
	fn from(value: BuildSkillShape) -> Self {
		match value {
			BuildSkillShape::GroundTargetedAoe(v) => {
				BuildSkillShapeDto::GroundTargetedAoe(SpawnGroundTargetedAoe::from(v))
			}
			BuildSkillShape::Projectile(v) => BuildSkillShapeDto::Projectile(v),
			BuildSkillShape::Shield(v) => BuildSkillShapeDto::Shield(v),
			#[cfg(test)]
			BuildSkillShape::Fn(_) => panic!("FN CANNOT BE SERIALIZED"),
		}
	}
}
