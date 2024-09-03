use crate::{
	behaviors::{SkillCaster, SkillSpawner, Target},
	traits::skill_builder::{BuildContact, BuildProjection, SkillLifetime},
};
use behaviors::components::{ground_targeted_aoe::GroundTargetedAoe, Contact, Projection};
use bevy::{
	prelude::{Bundle, Transform},
	utils::default,
};
use common::tools::Units;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Default, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum LifeTimeData {
	#[default]
	UntilStopped,
	UntilOutlived(Duration),
}

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnGroundTargetedAoe {
	pub lifetime: LifeTimeData,
	pub max_range: Units,
	pub radius: Units,
}

impl BuildContact for SpawnGroundTargetedAoe {
	fn build_contact(
		&self,
		caster: &SkillCaster,
		_: &SkillSpawner,
		target: &Target,
	) -> impl Bundle {
		let SkillCaster(.., caster_transform) = caster;
		let Target { ray, .. } = target;

		GroundTargetedAoe::<Contact> {
			caster: Transform::from(*caster_transform),
			target_ray: *ray,
			max_range: self.max_range,
			radius: self.radius,
			..default()
		}
	}
}

impl BuildProjection for SpawnGroundTargetedAoe {
	fn build_projection(
		&self,
		caster: &SkillCaster,
		_: &SkillSpawner,
		target: &Target,
	) -> impl Bundle {
		let SkillCaster(.., caster_transform) = caster;
		let Target { ray, .. } = target;

		GroundTargetedAoe::<Projection> {
			caster: Transform::from(*caster_transform),
			target_ray: *ray,
			max_range: self.max_range,
			radius: self.radius,
			..default()
		}
	}
}

impl SkillLifetime for SpawnGroundTargetedAoe {
	fn lifetime(&self) -> LifeTimeData {
		self.lifetime
	}
}
