use super::Blockers;
use crate::{
	behaviors::SkillCaster,
	components::SkillTarget,
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{SkillLifetime, SpawnShape},
};
use common::{
	tools::Units,
	traits::{
		clamp_zero_positive::ClampZeroPositive,
		handles_skill_behaviors::{
			Contact,
			HandlesSkillBehaviors,
			Motion,
			Projection,
			Shape,
			SkillEntities,
			SkillSpawner,
		},
	},
	zyheeda_commands::ZyheedaCommands,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnBeam {
	range: Units,
	blocked_by: Blockers,
}

impl SpawnShape for SpawnBeam {
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
				shape: Shape::Beam {
					range: self.range,
					radius: Units::new(0.003),
					blocked_by: self.blocked_by.clone().into(),
				},
				motion: Motion::HeldBy { caster, spawner },
			},
			Projection {
				shape: Shape::Beam {
					range: Units::new(1.0),
					radius: Units::new(0.2),
					blocked_by: HashSet::default(),
				},
				offset: None,
			},
		)
	}
}

impl SkillLifetime for SpawnBeam {
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::UntilStopped
	}
}
