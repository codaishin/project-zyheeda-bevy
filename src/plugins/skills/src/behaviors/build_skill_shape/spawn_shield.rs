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
	Offset,
	Shape,
	SkillTarget,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;
use common::{
	components::AssetModel,
	tools::Units,
	traits::clamp_zero_positive::ClampZeroPositive,
};
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
	type TProjection = SkillProjection;

	fn build_projection(
		&self,
		_: &SkillCaster,
		_: &SkillSpawner,
		_: &SkillTarget,
	) -> Self::TProjection {
		let radius = 1.;
		let offset = Vec3::new(0., 0., radius);

		SkillProjection {
			shape: Shape::Sphere {
				radius: Units::new(radius),
				hollow_collider: false,
			},
			offset: Some(Offset(offset)),
		}
	}
}

impl SkillLifetime for SpawnShield {
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::UntilStopped
	}
}
