use crate::{
	behaviors::{SkillCaster, SkillSpawner},
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{BuildContact, BuildProjection, SkillLifetime},
};
use behaviors::components::{
	shield::{ShieldContact, ShieldProjection},
	skill_behavior::SkillTarget,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnShield;

impl BuildContact for SpawnShield {
	fn build_contact(
		&self,
		_: &SkillCaster,
		spawner: &SkillSpawner,
		_: &SkillTarget,
	) -> impl Bundle {
		let SkillSpawner(location) = *spawner;

		(ShieldContact { location }, SpatialBundle::default())
	}
}

impl BuildProjection for SpawnShield {
	fn build_projection(&self, _: &SkillCaster, _: &SkillSpawner, _: &SkillTarget) -> impl Bundle {
		ShieldProjection
	}
}

impl SkillLifetime for SpawnShield {
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::UntilStopped
	}
}
