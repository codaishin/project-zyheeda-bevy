use crate::behaviors::skill_shape::{
	SkillShape,
	beam::Beam,
	ground_target::GroundTargetedAoe,
	projectile::Projectile,
	shield::Shield,
};
use common::dto::duration_in_seconds::DurationInSeconds;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) enum SpawnSkillDto {
	GroundTargetedAoe(GroundTargetedAoe<DurationInSeconds>),
	Projectile(Projectile),
	Beam(Beam),
	Shield(Shield),
}

impl From<SpawnSkillDto> for SkillShape {
	fn from(value: SpawnSkillDto) -> Self {
		match value {
			SpawnSkillDto::GroundTargetedAoe(v) => Self::GroundTargetedAoe(v.into()),
			SpawnSkillDto::Projectile(v) => Self::Projectile(v),
			SpawnSkillDto::Shield(v) => Self::Shield(v),
			SpawnSkillDto::Beam(v) => Self::Beam(v),
		}
	}
}

impl From<SkillShape> for SpawnSkillDto {
	fn from(value: SkillShape) -> Self {
		match value {
			SkillShape::GroundTargetedAoe(v) => Self::GroundTargetedAoe(v.into()),
			SkillShape::Projectile(v) => Self::Projectile(v),
			SkillShape::Shield(v) => Self::Shield(v),
			SkillShape::Beam(v) => Self::Beam(v),
			#[cfg(test)]
			SkillShape::Fn(_) => panic!("FN CANNOT BE SERIALIZED"),
		}
	}
}
