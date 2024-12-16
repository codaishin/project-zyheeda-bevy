use crate::{
	behaviors::{SkillCaster, SkillSpawner, SkillTarget},
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{BuildContact, BuildProjection, SkillLifetime},
};
use behaviors::components::skill_behavior::{
	skill_contact::SkillContact,
	skill_projection::SkillProjection,
	Integrity,
	Motion,
	Shape,
};
use common::{dto::duration::DurationDto, tools::Units};
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
	type TContact = SkillContact;

	fn build_contact(
		&self,
		caster: &SkillCaster,
		_: &SkillSpawner,
		target: &SkillTarget,
	) -> Self::TContact {
		let SkillCaster(caster) = *caster;
		let SkillTarget { ray, .. } = target;

		SkillContact {
			shape: Shape::Sphere {
				radius: self.radius,
				hollow_collider: true,
			},
			integrity: Integrity::Solid,
			motion: Motion::Stationary {
				caster,
				max_cast_range: self.max_range,
				target_ray: *ray,
			},
		}
	}
}

impl BuildProjection for SpawnGroundTargetedAoe {
	type TProjection = SkillProjection;

	fn build_projection(
		&self,
		_: &SkillCaster,
		_: &SkillSpawner,
		_: &SkillTarget,
	) -> Self::TProjection {
		SkillProjection {
			shape: Shape::Sphere {
				radius: self.radius,
				hollow_collider: false,
			},
			offset: None,
		}
	}
}

impl SkillLifetime for SpawnGroundTargetedAoe {
	fn lifetime(&self) -> LifeTimeDefinition {
		self.lifetime
	}
}
