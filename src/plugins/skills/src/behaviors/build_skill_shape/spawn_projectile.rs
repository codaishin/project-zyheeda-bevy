use crate::{
	behaviors::{SkillCaster, SkillSpawner},
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{BuildContact, BuildProjection, SkillLifetime},
};
use behaviors::components::{
	projectile::{sub_type::SubType, ProjectileContact, ProjectileProjection},
	skill_behavior::SkillTarget,
};
use bevy::prelude::{Bundle, SpatialBundle};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnProjectile {
	sub_type: SubType,
}

impl BuildContact for SpawnProjectile {
	fn build_contact(
		&self,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		_: &SkillTarget,
	) -> impl Bundle {
		let SkillCaster(caster) = *caster;
		let SkillSpawner(spawner) = *spawner;

		(
			ProjectileContact {
				caster,
				spawner,
				range: 10.,
				sub_type: self.sub_type,
			},
			SpatialBundle::default(),
		)
	}
}

impl BuildProjection for SpawnProjectile {
	fn build_projection(&self, _: &SkillCaster, _: &SkillSpawner, _: &SkillTarget) -> impl Bundle {
		ProjectileProjection {
			sub_type: self.sub_type,
		}
	}
}

impl SkillLifetime for SpawnProjectile {
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::Infinite
	}
}
