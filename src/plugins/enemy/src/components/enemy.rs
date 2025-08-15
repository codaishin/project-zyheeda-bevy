use crate::components::void_sphere::{VoidSphere, VoidSphereSlot};
use bevy::{asset::AssetPath, prelude::*};
use common::{
	components::{
		collider_relationship::InteractionTarget,
		is_blocker::{Blocker, IsBlocker},
		persistent_entity::PersistentEntity,
	},
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
		handles_enemies::{EnemySkillUsage, EnemyTarget},
		handles_skill_behaviors::SkillSpawner,
		loadout::LoadoutConfig,
		mapper::Mapper,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot, VisibleSlots},
	},
};
use std::time::Duration;

#[derive(Component, Clone)]
#[require(
	InteractionTarget,
	PersistentEntity,
	Transform,
	Visibility,
	IsBlocker = [Blocker::Character],
)]
pub struct Enemy {
	pub(crate) speed: Speed,
	pub(crate) movement_animation: Option<MovementAnimation>,
	pub(crate) aggro_range: AggroRange,
	pub(crate) attack_range: AttackRange,
	pub(crate) target: EnemyTarget,
	pub(crate) collider_radius: ColliderRadius,
}

impl From<&Enemy> for Speed {
	fn from(enemy: &Enemy) -> Self {
		enemy.speed
	}
}

impl<'a> From<&'a Enemy> for Option<&'a MovementAnimation> {
	fn from(enemy: &'a Enemy) -> Self {
		enemy.movement_animation.as_ref()
	}
}

impl From<&Enemy> for AggroRange {
	fn from(enemy: &Enemy) -> Self {
		enemy.aggro_range
	}
}

impl From<&Enemy> for AttackRange {
	fn from(enemy: &Enemy) -> Self {
		enemy.attack_range
	}
}

impl From<&Enemy> for EnemyTarget {
	fn from(enemy: &Enemy) -> Self {
		enemy.target
	}
}

impl From<&Enemy> for ColliderRadius {
	fn from(enemy: &Enemy) -> Self {
		enemy.collider_radius
	}
}

impl LoadoutConfig for Enemy {
	fn inventory(&self) -> impl Iterator<Item = Option<AssetPath<'static>>> {
		VoidSphere.inventory()
	}

	fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<AssetPath<'static>>)> {
		VoidSphere.slots()
	}
}

impl VisibleSlots for Enemy {
	fn visible_slots(&self) -> impl Iterator<Item = SlotKey> {
		[SlotKey::from(VoidSphereSlot)].into_iter()
	}
}

impl Mapper<Bone<'_>, Option<EssenceSlot>> for Enemy {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<EssenceSlot> {
		if bone != VoidSphere::SKILL_SPAWN {
			return None;
		}

		Some(EssenceSlot(SlotKey::from(VoidSphereSlot)))
	}
}

impl Mapper<Bone<'_>, Option<HandSlot>> for Enemy {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<HandSlot> {
		if bone != VoidSphere::SKILL_SPAWN {
			return None;
		}

		Some(HandSlot(SlotKey::from(VoidSphereSlot)))
	}
}

impl Mapper<Bone<'_>, Option<ForearmSlot>> for Enemy {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<ForearmSlot> {
		if bone != VoidSphere::SKILL_SPAWN {
			return None;
		}

		Some(ForearmSlot(SlotKey::from(VoidSphereSlot)))
	}
}

impl Mapper<Bone<'_>, Option<SkillSpawner>> for Enemy {
	fn map(&self, bone: Bone) -> Option<SkillSpawner> {
		VoidSphere.map(bone)
	}
}

impl EnemySkillUsage for Enemy {
	fn hold_skill(&self) -> Duration {
		VoidSphere.hold_skill()
	}

	fn cool_down(&self) -> Duration {
		VoidSphere.cool_down()
	}

	fn skill_key(&self) -> SlotKey {
		VoidSphere.skill_key()
	}
}
