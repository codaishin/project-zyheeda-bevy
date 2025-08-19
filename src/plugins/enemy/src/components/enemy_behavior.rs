use crate::{
	components::void_sphere::{VoidSphere, VoidSphereSlot},
	traits::insert_attack::InsertAttack,
};
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
		handles_enemies::{Attacker, EnemyAttack, EnemyTarget, Target},
		loadout::LoadoutConfig,
		mapper::Mapper,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot, VisibleSlots},
	},
};
use std::{sync::Arc, time::Duration};

#[derive(Component, Clone)]
#[require(
	InteractionTarget,
	PersistentEntity,
	Transform,
	Visibility,
	IsBlocker = [Blocker::Character],
)]
pub struct EnemyBehavior {
	pub(crate) speed: Speed,
	pub(crate) movement_animation: Option<MovementAnimation>,
	pub(crate) aggro_range: AggroRange,
	pub(crate) attack_range: AttackRange,
	pub(crate) target: EnemyTarget,
	pub(crate) attack: Arc<dyn InsertAttack + Sync + Send + 'static>,
	pub(crate) cool_down: Duration,
	pub(crate) collider_radius: ColliderRadius,
}

impl From<&EnemyBehavior> for Speed {
	fn from(enemy: &EnemyBehavior) -> Self {
		enemy.speed
	}
}

impl<'a> From<&'a EnemyBehavior> for Option<&'a MovementAnimation> {
	fn from(enemy: &'a EnemyBehavior) -> Self {
		enemy.movement_animation.as_ref()
	}
}

impl From<&EnemyBehavior> for AggroRange {
	fn from(enemy: &EnemyBehavior) -> Self {
		enemy.aggro_range
	}
}

impl From<&EnemyBehavior> for AttackRange {
	fn from(enemy: &EnemyBehavior) -> Self {
		enemy.attack_range
	}
}

impl From<&EnemyBehavior> for EnemyTarget {
	fn from(enemy: &EnemyBehavior) -> Self {
		enemy.target
	}
}

impl From<&EnemyBehavior> for ColliderRadius {
	fn from(enemy: &EnemyBehavior) -> Self {
		enemy.collider_radius
	}
}

impl LoadoutConfig for EnemyBehavior {
	fn inventory(&self) -> impl Iterator<Item = Option<AssetPath<'static>>> {
		std::iter::empty()
	}

	fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<AssetPath<'static>>)> {
		std::iter::once((SlotKey::from(VoidSphereSlot), None))
	}
}

impl VisibleSlots for EnemyBehavior {
	fn visible_slots(&self) -> impl Iterator<Item = SlotKey> {
		[SlotKey::from(VoidSphereSlot)].into_iter()
	}
}

impl Mapper<Bone<'_>, Option<EssenceSlot>> for EnemyBehavior {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<EssenceSlot> {
		if bone != VoidSphere::SLOT_NAME {
			return None;
		}

		Some(EssenceSlot(SlotKey::from(VoidSphereSlot)))
	}
}

impl Mapper<Bone<'_>, Option<HandSlot>> for EnemyBehavior {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<HandSlot> {
		if bone != VoidSphere::SLOT_NAME {
			return None;
		}

		Some(HandSlot(SlotKey::from(VoidSphereSlot)))
	}
}

impl Mapper<Bone<'_>, Option<ForearmSlot>> for EnemyBehavior {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<ForearmSlot> {
		if bone != VoidSphere::SLOT_NAME {
			return None;
		}

		Some(ForearmSlot(SlotKey::from(VoidSphereSlot)))
	}
}

impl EnemyAttack for EnemyBehavior {
	fn insert_attack(&self, entity: &mut EntityCommands, attacker: Attacker, target: Target) {
		self.attack.insert_attack(entity, attacker, target);
	}

	fn cool_down(&self) -> Duration {
		self.cool_down
	}
}
