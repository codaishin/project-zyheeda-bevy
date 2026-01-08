use crate::{
	behaviors::{
		Contact,
		SkillCaster,
		SkillContact,
		SkillLifetime,
		SkillProjection,
		skill_shape::Blockers,
	},
	skills::lifetime_definition::LifeTimeDefinition,
};
use common::{
	tools::Units,
	traits::handles_skill_physics::{
		ContactShape,
		Motion,
		Projection,
		ProjectionShape,
		SkillSpawner,
		SkillTarget,
	},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Beam {
	pub(crate) range: Units,
	pub(crate) blocked_by: Blockers,
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
