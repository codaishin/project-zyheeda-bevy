use crate::{
	behaviors::{SkillCaster, SkillSpawner, Target},
	traits::skill_builder::{BuildContact, BuildProjection, LifeTimeDefinition, SkillLifetime},
};
use behaviors::components::shield::{ShieldContact, ShieldProjection};
use bevy::prelude::{Bundle, SpatialBundle, Transform};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnShield;

impl BuildContact for SpawnShield {
	fn build_contact(&self, _: &SkillCaster, spawner: &SkillSpawner, _: &Target) -> impl Bundle {
		let SkillSpawner(entity, transform) = spawner;

		(
			ShieldContact { location: *entity },
			SpatialBundle::from_transform(Transform::from(*transform)),
		)
	}
}

impl BuildProjection for SpawnShield {
	fn build_projection(&self, _: &SkillCaster, _: &SkillSpawner, _: &Target) -> impl Bundle {
		ShieldProjection
	}
}

impl SkillLifetime for SpawnShield {
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::UntilStopped
	}
}
