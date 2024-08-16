use crate::behaviors::spawn_behavior::{
	spawn_ground_target::SpawnGroundTarget,
	spawn_projectile::SpawnProjectile,
	spawn_shield::SpawnShield,
	OnSkillStop,
	SpawnBehavior,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum SpawnBehaviorData {
	Placeholder, // Fixme: this is temporary, until we establish proper modularization of all skills
	GroundTarget(SpawnGroundTarget),
	Projectile(SpawnProjectile),
	Shield(SpawnShield),
}

impl From<SpawnBehaviorData> for SpawnBehavior {
	fn from(value: SpawnBehaviorData) -> Self {
		match value {
			SpawnBehaviorData::Placeholder => Self::Fn(|c, _, _, _| {
				let entity = c.spawn_empty();
				let id = entity.id();
				(entity, OnSkillStop::Stop(id))
			}),
			SpawnBehaviorData::GroundTarget(v) => Self::GroundTarget(v),
			SpawnBehaviorData::Projectile(v) => Self::Projectile(v),
			SpawnBehaviorData::Shield(v) => Self::Shield(v),
		}
	}
}
