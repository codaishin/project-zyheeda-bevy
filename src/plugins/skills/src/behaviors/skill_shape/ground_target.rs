use crate::{
	behaviors::SkillCaster,
	skills::lifetime_definition::LifeTimeDefinition,
	traits::{
		skill_builder::{SkillLifetime, SpawnShape},
		spawn_skill::{SkillContact, SkillProjection},
	},
};
use common::{
	dto::duration_in_seconds::DurationInSeconds,
	tools::Units,
	traits::{
		handles_physics::colliders::Blocker,
		handles_skill_physics::{
			Contact,
			ContactShape,
			HandlesNewPhysicalSkill,
			Motion,
			Projection,
			ProjectionShape,
			SkillEntities,
			SkillSpawner,
			SkillTarget,
		},
	},
	zyheeda_commands::ZyheedaCommands,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct GroundTargetedAoe<TDuration = Duration> {
	pub lifetime: LifeTimeDefinition<TDuration>,
	pub max_range: Units,
	pub radius: Units,
}

impl From<GroundTargetedAoe<DurationInSeconds>> for GroundTargetedAoe {
	fn from(with_lifetime_dto: GroundTargetedAoe<DurationInSeconds>) -> Self {
		Self {
			lifetime: with_lifetime_dto.lifetime.into(),
			max_range: with_lifetime_dto.max_range,
			radius: with_lifetime_dto.radius,
		}
	}
}

impl From<GroundTargetedAoe> for GroundTargetedAoe<DurationInSeconds> {
	fn from(with_lifetime_duration: GroundTargetedAoe) -> Self {
		Self {
			lifetime: with_lifetime_duration.lifetime.into(),
			max_range: with_lifetime_duration.max_range,
			radius: with_lifetime_duration.radius,
		}
	}
}

impl SpawnShape for GroundTargetedAoe {
	fn spawn_shape<TSkillBehaviors>(
		&self,
		commands: &mut ZyheedaCommands,
		caster: SkillCaster,
		_: SkillSpawner,
		target: SkillTarget,
	) -> SkillEntities
	where
		TSkillBehaviors: HandlesNewPhysicalSkill + 'static,
	{
		TSkillBehaviors::spawn_skill(
			commands,
			Contact {
				shape: ContactShape::Sphere {
					radius: self.radius,
					hollow_collider: true,
					destroyed_by: Blocker::none(),
				},
				motion: Motion::Stationary {
					caster,
					max_cast_range: self.max_range,
					target,
				},
			},
			Projection {
				shape: ProjectionShape::Sphere {
					radius: self.radius,
				},
				offset: None,
			},
		)
	}
}

impl SkillContact for GroundTargetedAoe {
	fn skill_contact(&self, caster: SkillCaster, _: SkillSpawner, target: SkillTarget) -> Contact {
		Contact {
			shape: ContactShape::Sphere {
				radius: self.radius,
				hollow_collider: true,
				destroyed_by: Blocker::none(),
			},
			motion: Motion::Stationary {
				caster,
				max_cast_range: self.max_range,
				target,
			},
		}
	}
}

impl SkillProjection for GroundTargetedAoe {
	fn skill_projection(&self) -> Projection {
		Projection {
			shape: ProjectionShape::Sphere {
				radius: self.radius,
			},
			offset: None,
		}
	}
}

impl SkillLifetime for GroundTargetedAoe {
	fn lifetime(&self) -> LifeTimeDefinition {
		self.lifetime
	}
}
