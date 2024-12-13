use crate::{
	behaviors::{SkillCaster, SkillSpawner},
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{BuildContact, BuildProjection, SkillLifetime},
};
use behaviors::components::skill_behavior::{
	skill_contact::SkillContact,
	skill_projection::SkillProjection,
	Integrity,
	Motion,
	Shape,
	SkillTarget,
};
use common::{
	blocker::Blocker,
	tools::{Units, UnitsPerSecond},
	traits::clamp_zero_positive::ClampZeroPositive,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnProjectile;

impl BuildContact for SpawnProjectile {
	type TContact = SkillContact;

	fn build_contact(
		&self,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		_: &SkillTarget,
	) -> Self::TContact {
		let SkillCaster(caster) = *caster;
		let SkillSpawner(spawner) = *spawner;

		SkillContact {
			shape: Shape::Sphere {
				radius: Units::new(0.05),
				hollow_collider: false,
			},
			integrity: Integrity::Fragile {
				destroyed_by: vec![Blocker::Physical, Blocker::Force],
			},
			motion: Motion::Projectile {
				caster,
				spawner,
				speed: UnitsPerSecond::new(15.),
				max_range: Units::new(20.),
			},
		}
	}
}

impl BuildProjection for SpawnProjectile {
	type TProjection = SkillProjection;

	fn build_projection(
		&self,
		_: &SkillCaster,
		_: &SkillSpawner,
		_: &SkillTarget,
	) -> Self::TProjection {
		SkillProjection {
			shape: Shape::Sphere {
				radius: Units::new(0.5),
				hollow_collider: false,
			},
			offset: None,
		}
	}
}

impl SkillLifetime for SpawnProjectile {
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::Infinite
	}
}
