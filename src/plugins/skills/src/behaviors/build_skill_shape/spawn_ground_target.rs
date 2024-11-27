use std::time::Duration;

use crate::{
	behaviors::{SkillCaster, SkillSpawner, Target},
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{BuildContact, BuildProjection, SkillLifetime},
};
use behaviors::components::ground_targeted_aoe::{
	GroundTargetedAoeContact,
	GroundTargetedAoeProjection,
};
use bevy::prelude::*;
use common::{dto::duration::DurationDto, tools::Units};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnGroundTargetedAoe<TDuration = Duration> {
	pub lifetime: LifeTimeDefinition<TDuration>,
	pub max_range: Units,
	pub radius: Units,
}

impl From<SpawnGroundTargetedAoe<DurationDto>> for SpawnGroundTargetedAoe {
	fn from(with_lifetime_dto: SpawnGroundTargetedAoe<DurationDto>) -> Self {
		Self {
			lifetime: with_lifetime_dto.lifetime.into(),
			max_range: with_lifetime_dto.max_range,
			radius: with_lifetime_dto.radius,
		}
	}
}

impl BuildContact for SpawnGroundTargetedAoe {
	fn build_contact(
		&self,
		caster: &SkillCaster,
		_: &SkillSpawner,
		target: &Target,
	) -> impl Bundle {
		let SkillCaster(caster) = *caster;
		let Target { ray, .. } = target;

		GroundTargetedAoeContact {
			caster,
			target_ray: *ray,
			max_range: self.max_range,
			radius: self.radius,
		}
	}
}

impl BuildProjection for SpawnGroundTargetedAoe {
	fn build_projection(&self, _: &SkillCaster, _: &SkillSpawner, _: &Target) -> impl Bundle {
		GroundTargetedAoeProjection {
			radius: self.radius,
		}
	}
}

impl SkillLifetime for SpawnGroundTargetedAoe {
	fn lifetime(&self) -> LifeTimeDefinition {
		self.lifetime
	}
}
