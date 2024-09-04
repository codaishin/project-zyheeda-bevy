use crate::{
	behaviors::{SkillCaster, SkillSpawner, Target},
	traits::skill_builder::{BuildContact, BuildProjection, LifeTimeDefinition, SkillLifetime},
};
use behaviors::components::ground_targeted_aoe::{
	GroundTargetedAoeContact,
	GroundTargetedAoeProjection,
};
use bevy::prelude::{Bundle, Transform};
use common::tools::Units;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnGroundTargetedAoe<TLifeTime> {
	pub lifetime: TLifeTime,
	pub max_range: Units,
	pub radius: Units,
}

impl<T> BuildContact for SpawnGroundTargetedAoe<T> {
	fn build_contact(
		&self,
		caster: &SkillCaster,
		_: &SkillSpawner,
		target: &Target,
	) -> impl Bundle {
		let SkillCaster(.., caster_transform) = caster;
		let Target { ray, .. } = target;

		GroundTargetedAoeContact {
			caster: Transform::from(*caster_transform),
			target_ray: *ray,
			max_range: self.max_range,
			radius: self.radius,
		}
	}
}

impl<T> BuildProjection for SpawnGroundTargetedAoe<T> {
	fn build_projection(&self, _: &SkillCaster, _: &SkillSpawner, _: &Target) -> impl Bundle {
		GroundTargetedAoeProjection {
			radius: self.radius,
		}
	}
}

impl<T> SkillLifetime for SpawnGroundTargetedAoe<T>
where
	LifeTimeDefinition: From<T>,
	T: Clone,
{
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::from(self.lifetime.clone())
	}
}
