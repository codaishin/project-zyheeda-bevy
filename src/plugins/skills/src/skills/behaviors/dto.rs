use crate::skills::{
	behaviors::SkillBehaviorConfig,
	shape::{
		SkillShape,
		SpawnOn,
		beam::Beam,
		ground_target::GroundTargetedAoe,
		projectile::Projectile,
		shield::Shield,
	},
};
use common::{dto::duration_in_seconds::DurationInSeconds, traits::handles_skill_physics::Effect};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct SkillBehaviorConfigDto {
	shape: SpawnSkillDto,
	contact: Vec<Effect>,
	projection: Vec<Effect>,
	spawn_on: SpawnOn,
}

impl From<SkillBehaviorConfigDto> for SkillBehaviorConfig {
	fn from(value: SkillBehaviorConfigDto) -> Self {
		Self {
			shape: SkillShape::from(value.shape),
			contact: value.contact,
			projection: value.projection,
			spawn_on: value.spawn_on,
		}
	}
}

impl From<SkillBehaviorConfig> for SkillBehaviorConfigDto {
	fn from(value: SkillBehaviorConfig) -> Self {
		Self {
			shape: SpawnSkillDto::from(value.shape),
			contact: value.contact,
			projection: value.projection,
			spawn_on: value.spawn_on,
		}
	}
}

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
		}
	}
}
