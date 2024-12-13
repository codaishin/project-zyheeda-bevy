use crate::{
	behaviors::{SkillCaster, SkillSpawner},
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{BuildContact, BuildProjection, SkillLifetime},
};
use behaviors::components::{
	shield::ShieldProjection,
	skill_behavior::{skill_contact::SkillContact, Integrity, Motion, Shape, SkillTarget},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;
use common::components::AssetModel;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnShield;

impl BuildContact for SpawnShield {
	type TContact = SkillContact;

	fn build_contact(
		&self,
		_: &SkillCaster,
		spawner: &SkillSpawner,
		_: &SkillTarget,
	) -> Self::TContact {
		let SkillSpawner(spawner) = *spawner;

		SkillContact {
			shape: Shape::Custom {
				model: AssetModel::path("models/shield.glb"),
				collider: Collider::cuboid(0.5, 0.5, 0.05),
				scale: Vec3::splat(1.),
			},
			integrity: Integrity::Solid,
			motion: Motion::HeldBy { spawner },
		}
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
