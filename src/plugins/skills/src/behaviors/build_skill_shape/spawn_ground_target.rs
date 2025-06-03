use crate::{
	behaviors::SkillCaster,
	components::SkillTarget,
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{BuildContact, BuildProjection, SkillLifetime},
};
use common::{
	dto::duration::DurationDto,
	tools::Units,
	traits::handles_skill_behaviors::{HandlesSkillBehaviors, Integrity, Motion, Shape, Spawner},
};
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
	fn build_contact<TSkillBehaviors>(
		&self,
		caster: &SkillCaster,
		_: Spawner,
		target: &SkillTarget,
	) -> TSkillBehaviors::TSkillContact
	where
		TSkillBehaviors: HandlesSkillBehaviors,
	{
		let SkillCaster(caster) = *caster;
		let SkillTarget { ray, .. } = target;

		TSkillBehaviors::skill_contact(
			Shape::Sphere {
				radius: self.radius,
				hollow_collider: true,
			},
			Integrity::Solid,
			Motion::Stationary {
				caster,
				max_cast_range: self.max_range,
				target_ray: *ray,
			},
		)
	}
}

impl BuildProjection for SpawnGroundTargetedAoe {
	fn build_projection<TSkillBehaviors>(
		&self,
		_: &SkillCaster,
		_: Spawner,
		_: &SkillTarget,
	) -> TSkillBehaviors::TSkillProjection
	where
		TSkillBehaviors: HandlesSkillBehaviors,
	{
		TSkillBehaviors::skill_projection(
			Shape::Sphere {
				radius: self.radius,
				hollow_collider: false,
			},
			None,
		)
	}
}

impl SkillLifetime for SpawnGroundTargetedAoe {
	fn lifetime(&self) -> LifeTimeDefinition {
		self.lifetime
	}
}
