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
	tools::Units,
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
pub struct Beam {
	range: Units,
	blocked_by: Blockers,
}

impl SpawnShape for Beam {
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
				shape: ContactShape::Beam {
					range: self.range,
					radius: Units::from(0.003),
					blocked_by: self.blocked_by.clone().into(),
				},
				motion: Motion::HeldBy { caster, spawner },
			},
			Projection {
				shape: ProjectionShape::Beam {
					radius: Units::from(0.2),
				},
				offset: None,
			},
		)
	}
}

impl SkillContact for Beam {
	fn skill_contact(&self, caster: SkillCaster, spawner: SkillSpawner, _: SkillTarget) -> Contact {
		Contact {
			shape: ContactShape::Beam {
				range: self.range,
				radius: Units::from(0.003),
				blocked_by: self.blocked_by.clone().into(),
			},
			motion: Motion::HeldBy { caster, spawner },
		}
	}
}

impl SkillProjection for Beam {
	fn skill_projection(&self) -> Projection {
		Projection {
			shape: ProjectionShape::Beam {
				radius: Units::from(0.2),
			},
			offset: None,
		}
	}
}

impl SkillLifetime for Beam {
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::UntilStopped
	}
}
