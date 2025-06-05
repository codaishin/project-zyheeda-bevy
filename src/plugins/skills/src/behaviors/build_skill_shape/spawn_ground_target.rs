use crate::{
	behaviors::SkillCaster,
	components::SkillTarget,
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{SkillLifetime, SpawnShape},
};
use bevy::prelude::*;
use common::{
	dto::duration::DurationDto,
	tools::Units,
	traits::handles_skill_behaviors::{
		Contact,
		HandlesSkillBehaviors,
		Integrity,
		Motion,
		Projection,
		Shape,
		SkillEntities,
		Spawner,
	},
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

impl SpawnShape for SpawnGroundTargetedAoe {
	fn spawn_shape<TSkillBehaviors>(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		_: Spawner,
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
				integrity: Integrity::Solid,
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
