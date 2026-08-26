use crate::skills::behaviors::SkillBehaviorConfig;
use common::prelude::*;
use macros::serde_model;

#[serde_model]
#[derive(Debug, PartialEq)]
pub(crate) struct SkillBehaviorConfigDto {
	shape: SpawnSkillDto,
	contact: Vec<SkillEffect>,
	projection: Vec<SkillEffect>,
}

impl From<SkillBehaviorConfigDto> for SkillBehaviorConfig {
	fn from(value: SkillBehaviorConfigDto) -> Self {
		Self {
			shape: SkillShape::from(value.shape),
			contact: value.contact,
			projection: value.projection,
		}
	}
}

impl From<SkillBehaviorConfig> for SkillBehaviorConfigDto {
	fn from(value: SkillBehaviorConfig) -> Self {
		Self {
			shape: SpawnSkillDto::from(value.shape),
			contact: value.contact,
			projection: value.projection,
		}
	}
}

#[serde_model]
#[derive(Debug, PartialEq)]
pub(crate) enum SpawnSkillDto {
	GroundTargetedAoe(SphereAoE<DurationInSeconds>),
	Projectile(Projectile),
	Beam(Beam),
	Shield(Shield),
}

impl From<SpawnSkillDto> for SkillShape {
	fn from(value: SpawnSkillDto) -> Self {
		match value {
			SpawnSkillDto::GroundTargetedAoe(v) => Self::SphereAoE(v.into()),
			SpawnSkillDto::Projectile(v) => Self::Projectile(v),
			SpawnSkillDto::Shield(v) => Self::Shield(v),
			SpawnSkillDto::Beam(v) => Self::Beam(v),
		}
	}
}

impl From<SkillShape> for SpawnSkillDto {
	fn from(value: SkillShape) -> Self {
		match value {
			SkillShape::SphereAoE(v) => Self::GroundTargetedAoe(v.into()),
			SkillShape::Projectile(v) => Self::Projectile(v),
			SkillShape::Shield(v) => Self::Shield(v),
			SkillShape::Beam(v) => Self::Beam(v),
		}
	}
}
