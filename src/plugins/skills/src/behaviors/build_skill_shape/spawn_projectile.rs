use crate::{
	behaviors::{SkillCaster, SkillSpawner},
	components::SkillTarget,
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{BuildContact, BuildProjection, SkillLifetime},
};
use common::{
	blocker::Blocker,
	tools::{Units, UnitsPerSecond},
	traits::{
		clamp_zero_positive::ClampZeroPositive,
		handles_skill_behaviors::{HandlesSkillBehaviors, Integrity, Motion, Shape},
	},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnProjectile;

impl BuildContact for SpawnProjectile {
	fn build_contact<TSkillBehaviors>(
		&self,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		_: &SkillTarget,
	) -> TSkillBehaviors::TSkillContact
	where
		TSkillBehaviors: HandlesSkillBehaviors,
	{
		let SkillCaster(caster) = *caster;
		let SkillSpawner(spawner) = *spawner;

		TSkillBehaviors::skill_contact(
			Shape::Sphere {
				radius: Units::new(0.05),
				hollow_collider: false,
			},
			Integrity::Fragile {
				destroyed_by: vec![Blocker::Physical, Blocker::Force],
			},
			Motion::Projectile {
				caster,
				spawner,
				speed: UnitsPerSecond::new(15.),
				max_range: Units::new(20.),
			},
		)
	}
}

impl BuildProjection for SpawnProjectile {
	fn build_projection<TSkillBehaviors>(
		&self,
		_: &SkillCaster,
		_: &SkillSpawner,
		_: &SkillTarget,
	) -> TSkillBehaviors::TSkillProjection
	where
		TSkillBehaviors: HandlesSkillBehaviors,
	{
		TSkillBehaviors::skill_projection(
			Shape::Sphere {
				radius: Units::new(0.5),
				hollow_collider: false,
			},
			None,
		)
	}
}

impl SkillLifetime for SpawnProjectile {
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::Infinite
	}
}
