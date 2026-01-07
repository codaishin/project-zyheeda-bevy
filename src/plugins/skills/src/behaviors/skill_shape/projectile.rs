use super::Blockers;
use crate::{
	behaviors::SkillCaster,
	skills::lifetime_definition::LifeTimeDefinition,
	traits::{
		skill_builder::{SkillLifetime, SpawnShape},
		spawn_skill::{SkillContact, SkillProjection},
	},
};
use common::{
	tools::{Units, UnitsPerSecond},
	traits::handles_skill_physics::{
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
	zyheeda_commands::ZyheedaCommands,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Projectile {
	destroyed_by: Blockers,
}

impl SpawnShape for Projectile {
	fn spawn_shape<TSkillBehaviors>(
		&self,
		commands: &mut ZyheedaCommands,
		caster: SkillCaster,
		spawner: SkillSpawner,
		_: SkillTarget,
	) -> SkillEntities
	where
		TSkillBehaviors: HandlesNewPhysicalSkill + 'static,
	{
		TSkillBehaviors::spawn_skill(
			commands,
			Contact {
				shape: ContactShape::Sphere {
					radius: Units::from(0.05),
					hollow_collider: false,
					destroyed_by: self.destroyed_by.clone().into(),
				},
				motion: Motion::Projectile {
					caster,
					spawner,
					speed: UnitsPerSecond::from(15.),
					range: Units::from(20.),
				},
			},
			Projection {
				shape: ProjectionShape::Sphere {
					radius: Units::from(0.5),
				},
				offset: None,
			},
		)
	}
}

impl SkillContact for Projectile {
	fn skill_contact(&self, caster: SkillCaster, spawner: SkillSpawner, _: SkillTarget) -> Contact {
		Contact {
			shape: ContactShape::Sphere {
				radius: Units::from(0.05),
				hollow_collider: false,
				destroyed_by: self.destroyed_by.clone().into(),
			},
			motion: Motion::Projectile {
				caster,
				spawner,
				speed: UnitsPerSecond::from(15.),
				range: Units::from(20.),
			},
		}
	}
}

impl SkillProjection for Projectile {
	fn skill_projection(&self) -> Projection {
		Projection {
			shape: ProjectionShape::Sphere {
				radius: Units::from(0.5),
			},
			offset: None,
		}
	}
}

impl SkillLifetime for Projectile {
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::Infinite
	}
}
