use super::Blockers;
use crate::skills::{
	behaviors::{SkillContact, SkillLifetime, SkillProjection},
	lifetime_definition::LifeTimeDefinition,
};
use common::{
	tools::{Units, UnitsPerSecond},
	traits::handles_skill_physics::{
		Contact,
		ContactShape,
		Motion,
		Projection,
		ProjectionShape,
		SkillCaster,
		SkillSpawner,
		SkillTarget,
	},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Projectile {
	pub(crate) destroyed_by: Blockers,
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
