use super::accessors::get::RefInto;
use crate::{
	components::persistent_entity::PersistentEntity,
	tools::{
		action_key::slot::SlotKey,
		aggro_range::AggroRange,
		attack_range::AttackRange,
		bone::Bone,
		collider_radius::ColliderRadius,
		movement_animation::MovementAnimation,
		speed::Speed,
	},
	traits::{
		handles_skill_behaviors::SkillSpawner,
		loadout::LoadoutConfig,
		mapper::Mapper,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot, VisibleSlots},
	},
};
use bevy::prelude::*;
use std::time::Duration;

pub trait HandlesEnemies {
	/// FIXME: Impl a `TEnemy::from(EnemyKind::VoidSphere)` pattern
	type TEnemy: Component + Default;
}

pub trait HandlesEnemyConfig {
	type TEnemyBehavior: Component
		+ LoadoutConfig
		+ VisibleSlots
		+ EnemySkillUsage
		+ for<'a> RefInto<'a, Speed>
		+ for<'a> RefInto<'a, Option<&'a MovementAnimation>>
		+ for<'a> RefInto<'a, EnemyTarget>
		+ for<'a> RefInto<'a, AggroRange>
		+ for<'a> RefInto<'a, AttackRange>
		+ for<'a> RefInto<'a, ColliderRadius>
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

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum EnemyTarget {
	#[default]
	Player,
	Entity(PersistentEntity),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Attacker(pub PersistentEntity);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Target(pub PersistentEntity);
