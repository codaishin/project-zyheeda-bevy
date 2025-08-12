use crate::{
	behaviors::SkillCaster,
	components::SkillTarget,
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{SkillLifetime, SpawnShape},
};
use common::{
	components::is_blocker::Blocker,
	tools::{Units, UnitsPerSecond},
	traits::{
		clamp_zero_positive::ClampZeroPositive,
		handles_skill_behaviors::{
			Contact,
			HandlesSkillBehaviors,
			Integrity,
			Motion,
			Projection,
			Shape,
			SkillEntities,
			SkillSpawner,
		},
	},
	zyheeda_commands::ZyheedaCommands,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnProjectile {
	destroyed_by: DestroyedBy,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
enum DestroyedBy {
	All,
	AnyOf(HashSet<Blocker>),
}

impl SpawnShape for SpawnProjectile {
	fn spawn_shape<TSkillBehaviors>(
		&self,
		commands: &mut ZyheedaCommands,
		caster: &SkillCaster,
		spawner: SkillSpawner,
		_: &SkillTarget,
	) -> SkillEntities
	where
		TSkillBehaviors: HandlesSkillBehaviors + 'static,
	{
		let SkillCaster(caster) = *caster;

		TSkillBehaviors::spawn_skill(
			commands,
			Contact {
				shape: Shape::Sphere {
					radius: Units::new(0.05),
					hollow_collider: false,
				},
				integrity: Integrity::Fragile {
					destroyed_by: match &self.destroyed_by {
						DestroyedBy::All => Blocker::all(),
						DestroyedBy::AnyOf(blockers) => blockers.clone(),
					},
				},
				motion: Motion::Projectile {
					caster,
					spawner,
					speed: UnitsPerSecond::new(15.),
					range: Units::new(20.),
				},
			},
			Projection {
				shape: Shape::Sphere {
					radius: Units::new(0.5),
					hollow_collider: false,
				},
				offset: None,
			},
		)
	}
}

impl SkillLifetime for SpawnProjectile {
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::Infinite
	}
}
