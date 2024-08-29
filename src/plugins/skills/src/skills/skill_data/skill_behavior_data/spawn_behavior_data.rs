use crate::behaviors::spawn_behavior::{
	spawn_ground_target::SpawnGroundTargetedAoe,
	spawn_projectile::SpawnProjectile,
	spawn_shield::SpawnShield,
	OnSkillStop,
	SpawnBehavior,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum SpawnBehaviorData<T: Sync + Send + 'static> {
	Placeholder, // Fixme: this is temporary, until we establish proper modularization of all skills
	GroundTargetedAoe(SpawnGroundTargetedAoe<T>),
	Projectile(SpawnProjectile),
	Shield(SpawnShield),
}

impl<T: Sync + Send + 'static> From<SpawnBehaviorData<T>> for SpawnBehavior<T> {
	fn from(value: SpawnBehaviorData<T>) -> Self {
		match value {
			SpawnBehaviorData::Placeholder => Self::Fn(|c, _, _, _| {
				let entity = c.spawn_empty();
				let id = entity.id();
				(entity, OnSkillStop::Stop(id))
			}),
			SpawnBehaviorData::GroundTargetedAoe(v) => Self::GroundTargetedAoe(v),
			SpawnBehaviorData::Projectile(v) => Self::Projectile(v),
			SpawnBehaviorData::Shield(v) => Self::Shield(v),
		}
	}
}
