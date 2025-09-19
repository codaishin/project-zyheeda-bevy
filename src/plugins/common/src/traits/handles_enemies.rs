use crate::{
	attributes::{effect_target::EffectTarget, health::Health},
	components::persistent_entity::PersistentEntity,
	effects::{force::Force, gravity::Gravity},
	tools::{
		action_key::slot::SlotKey,
		aggro_range::AggroRange,
		attack_range::AttackRange,
		attribute::AttributeOnSpawn,
		bone::Bone,
		collider_radius::ColliderRadius,
		movement_animation::MovementAnimation,
		speed::Speed,
	},
	traits::{
		accessors::get::{GetProperty, Property},
		handles_skill_behaviors::SkillSpawner,
		iteration::{Iter, IterFinite},
		loadout::LoadoutConfig,
		mapper::Mapper,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot, VisibleSlots},
	},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub trait HandlesEnemies {
	type TEnemy: Component
		+ From<EnemyType>
		+ LoadoutConfig
		+ VisibleSlots
		+ EnemySkillUsage
		+ GetProperty<Speed>
		+ GetProperty<Option<MovementAnimation>>
		+ GetProperty<EnemyTarget>
		+ GetProperty<AggroRange>
		+ GetProperty<AttackRange>
		+ GetProperty<ColliderRadius>
		+ GetProperty<AttributeOnSpawn<Health>>
		+ GetProperty<AttributeOnSpawn<EffectTarget<Gravity>>>
		+ GetProperty<AttributeOnSpawn<EffectTarget<Force>>>
		+ for<'a> Mapper<Bone<'a>, Option<EssenceSlot>>
		+ for<'a> Mapper<Bone<'a>, Option<HandSlot>>
		+ for<'a> Mapper<Bone<'a>, Option<ForearmSlot>>
		+ for<'a> Mapper<Bone<'a>, Option<SkillSpawner>>;
}

pub trait EnemySkillUsage {
	fn hold_skill(&self) -> Duration;
	fn cool_down(&self) -> Duration;
	fn skill_key(&self) -> SlotKey;
}

#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub enum EnemyTarget {
	#[default]
	Player,
	Entity(PersistentEntity),
}

impl Property for EnemyTarget {
	type TValue<'a> = Self;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Attacker(pub PersistentEntity);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Target(pub PersistentEntity);

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum EnemyType {
	VoidSphere,
}

impl IterFinite for EnemyType {
	fn iterator() -> Iter<Self> {
		Iter(Some(EnemyType::VoidSphere))
	}

	fn next(Iter(current): &Iter<Self>) -> Option<Self> {
		match current.as_ref()? {
			EnemyType::VoidSphere => None,
		}
	}
}

impl Property for EnemyType {
	type TValue<'a> = Self;
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_enemy_types_finite() {
		assert_eq!(
			vec![EnemyType::VoidSphere],
			EnemyType::iterator().collect::<Vec<_>>()
		);
	}
}
