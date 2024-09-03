use super::spawn_ground_target::LifeTimeData;
use crate::{
	behaviors::{SkillCaster, SkillSpawner, Target},
	traits::skill_builder::{BuildContact, BuildProjection, SkillLifetime},
};
use behaviors::components::projectile::{
	sub_type::SubType,
	ProjectileContact,
	ProjectileProjection,
};
use bevy::prelude::{Bundle, SpatialBundle, Transform};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnProjectile {
	sub_type: SubType,
}

impl BuildContact for SpawnProjectile {
	fn build_contact(
		&self,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		_: &Target,
	) -> impl Bundle {
		let SkillCaster(.., caster) = caster;
		let SkillSpawner(.., spawner) = spawner;

		(
			ProjectileContact {
				direction: caster.forward(),
				range: 10.,
				sub_type: self.sub_type,
			},
			SpatialBundle::from_transform(Transform::from(*spawner)),
		)
	}
}

impl BuildProjection for SpawnProjectile {
	fn build_projection(&self, _: &SkillCaster, _: &SkillSpawner, _: &Target) -> impl Bundle {
		ProjectileProjection {
			sub_type: self.sub_type,
		}
	}
}

impl SkillLifetime for SpawnProjectile {
	fn lifetime(&self) -> LifeTimeData {
		// FIXME: introduce infinite lifetime?
		LifeTimeData::UntilOutlived(Duration::MAX)
	}
}
