use super::Blockers;
use crate::{
	behaviors::SkillCaster,
	components::SkillTarget,
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{SkillLifetime, SpawnShape},
};
use common::{
	tools::{Units, UnitsPerSecond},
	traits::handles_skill_behaviors::{
		Contact,
		ContactShape,
		HandlesSkillBehaviors,
		Motion,
		Projection,
		ProjectionShape,
		SkillEntities,
		SkillSpawner,
	},
	zyheeda_commands::ZyheedaCommands,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnProjectile {
	destroyed_by: Blockers,
}

impl SpawnShape for SpawnProjectile {
	fn spawn_shape<TSkillBehaviors>(
		&self,
		commands: &mut ZyheedaCommands,
		caster: &SkillCaster,
		spawner: SkillSpawner,
		_: &SkillTarget,
	) -> SkillEntities
	where
		TSkillBehaviors: HandlesSkillBehaviors + 'static,
	{
		let SkillCaster(caster) = *caster;

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

impl SkillLifetime for SpawnProjectile {
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::Infinite
	}
}
