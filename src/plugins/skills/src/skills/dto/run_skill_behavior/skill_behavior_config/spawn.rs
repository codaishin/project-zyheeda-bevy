use crate::behaviors::spawn_skill::{
	SpawnSkill,
	spawn_beam::SpawnBeam,
	spawn_ground_target::SpawnGroundTargetedAoe,
	spawn_projectile::SpawnProjectile,
	spawn_shield::SpawnShield,
};
use common::dto::duration_secs_f32::DurationSecsF32;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) enum SpawnSkillDto {
	GroundTargetedAoe(SpawnGroundTargetedAoe<DurationSecsF32>),
	Projectile(SpawnProjectile),
	Beam(SpawnBeam),
	Shield(SpawnShield),
}

impl From<SpawnSkillDto> for SpawnSkill {
	fn from(value: SpawnSkillDto) -> Self {
		match value {
			SpawnSkillDto::GroundTargetedAoe(v) => Self::GroundTargetedAoe(v.into()),
			SpawnSkillDto::Projectile(v) => Self::Projectile(v),
			SpawnSkillDto::Shield(v) => Self::Shield(v),
			SpawnSkillDto::Beam(v) => Self::Beam(v),
		}
	}
}

impl From<SpawnSkill> for SpawnSkillDto {
	fn from(value: SpawnSkill) -> Self {
		match value {
			SpawnSkill::GroundTargetedAoe(v) => Self::GroundTargetedAoe(v.into()),
			SpawnSkill::Projectile(v) => Self::Projectile(v),
			SpawnSkill::Shield(v) => Self::Shield(v),
			SpawnSkill::Beam(v) => Self::Beam(v),
			#[cfg(test)]
			SpawnSkill::Fn(_) => panic!("FN CANNOT BE SERIALIZED"),
		}
	}
}
