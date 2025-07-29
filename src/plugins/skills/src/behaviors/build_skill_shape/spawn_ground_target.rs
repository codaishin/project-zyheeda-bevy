use crate::{
	behaviors::SkillCaster,
	components::SkillTarget,
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{SkillLifetime, SpawnShape},
};
use common::{
	dto::duration_secs_f32::DurationSecsF32,
	tools::Units,
	traits::handles_skill_behaviors::{
		Contact,
		HandlesSkillBehaviors,
		Motion,
		Projection,
		Shape,
		SkillEntities,
		SkillSpawner,
	},
	zyheeda_commands::ZyheedaCommands,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnGroundTargetedAoe<TDuration = Duration> {
	pub lifetime: LifeTimeDefinition<TDuration>,
	pub max_range: Units,
	pub radius: Units,
}

impl From<SpawnGroundTargetedAoe<DurationSecsF32>> for SpawnGroundTargetedAoe {
	fn from(with_lifetime_dto: SpawnGroundTargetedAoe<DurationSecsF32>) -> Self {
		Self {
			lifetime: with_lifetime_dto.lifetime.into(),
			max_range: with_lifetime_dto.max_range,
			radius: with_lifetime_dto.radius,
		}
	}
}

impl From<SpawnGroundTargetedAoe> for SpawnGroundTargetedAoe<DurationSecsF32> {
	fn from(with_lifetime_duration: SpawnGroundTargetedAoe) -> Self {
		Self {
			lifetime: with_lifetime_duration.lifetime.into(),
			max_range: with_lifetime_duration.max_range,
			radius: with_lifetime_duration.radius,
		}
	}
}

impl SpawnShape for SpawnGroundTargetedAoe {
	fn spawn_shape<TSkillBehaviors>(
		&self,
		commands: &mut ZyheedaCommands,
		caster: &SkillCaster,
		_: SkillSpawner,
		target: &SkillTarget,
	) -> SkillEntities
	where
		TSkillBehaviors: HandlesSkillBehaviors + 'static,
	{
		let SkillCaster(caster) = *caster;
		let SkillTarget { ray, .. } = target;

		TSkillBehaviors::spawn_skill(
			commands,
			Contact {
				shape: Shape::Sphere {
					radius: self.radius,
					hollow_collider: true,
				},
				motion: Motion::Stationary {
					caster,
					max_cast_range: self.max_range,
					target_ray: *ray,
				},
			},
			Projection {
				shape: Shape::Sphere {
					radius: self.radius,
					hollow_collider: false,
				},
				offset: None,
			},
		)
	}
}

impl SkillLifetime for SpawnGroundTargetedAoe {
	fn lifetime(&self) -> LifeTimeDefinition {
		self.lifetime
	}
}
