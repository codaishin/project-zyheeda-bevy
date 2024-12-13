use crate::{
	behaviors::{SkillCaster, SkillSpawner},
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{BuildContact, BuildProjection, SkillLifetime},
};
use behaviors::components::{
	projectile::ProjectileProjection,
	skill_behavior::{skill_contact::SkillContact, Integrity, Motion, Shape, SkillTarget},
};
use bevy::prelude::Bundle;
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
	fn build_projection(&self, _: &SkillCaster, _: &SkillSpawner, _: &SkillTarget) -> impl Bundle {
		ProjectileProjection
	}
}

impl SkillLifetime for SpawnProjectile {
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::Infinite
	}
}
