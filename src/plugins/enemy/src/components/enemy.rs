pub(crate) mod enemy_type;
pub(crate) mod void_sphere;

use crate::components::enemy::enemy_type::EnemyTypeInternal;
use bevy::{asset::AssetPath, prelude::*};
use bevy_rapier3d::prelude::{GravityScale, RigidBody};
use common::{
	components::{
		collider_relationship::InteractionTarget,
		ground_offset::GroundOffset,
		is_blocker::{Blocker, IsBlocker},
		persistent_entity::PersistentEntity,
	},
	errors::Error,
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
		handles_effects::HandlesAllEffects,
		handles_enemies::{EnemySkillUsage, EnemyTarget, EnemyType},
		handles_skill_behaviors::SkillSpawner,
		load_asset::LoadAsset,
		loadout::LoadoutConfig,
		mapper::Mapper,
		prefab::{Prefab, PrefabEntityCommands},
		register_derived_component::{DerivableFrom, InsertDerivedComponent},
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot, VisibleSlots},
	},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[require(
	InteractionTarget,
	PersistentEntity,
	Transform,
	Visibility,
	RigidBody = RigidBody::Dynamic,
	GravityScale = GravityScale(0.),
	IsBlocker = [Blocker::Character],
)]
pub struct Enemy {
	pub(crate) speed: Speed,
	pub(crate) movement_animation: Option<MovementAnimation>,
	pub(crate) aggro_range: AggroRange,
	pub(crate) attack_range: AttackRange,
	pub(crate) target: EnemyTarget,
	pub(crate) collider_radius: ColliderRadius,
	pub(crate) enemy_type: EnemyTypeInternal,
}

impl From<EnemyType> for Enemy {
	fn from(enemy_type: EnemyType) -> Self {
		EnemyTypeInternal::new_enemy(enemy_type)
	}
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
		self.enemy_type.inventory()
	}

	fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<AssetPath<'static>>)> {
		self.enemy_type.slots()
	}
}

impl VisibleSlots for Enemy {
	fn visible_slots(&self) -> impl Iterator<Item = SlotKey> {
		self.enemy_type.visible_slots()
	}
}

impl Mapper<Bone<'_>, Option<EssenceSlot>> for Enemy {
	fn map(&self, bone: Bone<'_>) -> Option<EssenceSlot> {
		self.enemy_type.map(bone)
	}
}

impl Mapper<Bone<'_>, Option<HandSlot>> for Enemy {
	fn map(&self, bone: Bone<'_>) -> Option<HandSlot> {
		self.enemy_type.map(bone)
	}
}

impl Mapper<Bone<'_>, Option<ForearmSlot>> for Enemy {
	fn map(&self, bone: Bone<'_>) -> Option<ForearmSlot> {
		self.enemy_type.map(bone)
	}
}

impl Mapper<Bone<'_>, Option<SkillSpawner>> for Enemy {
	fn map(&self, bone: Bone) -> Option<SkillSpawner> {
		self.enemy_type.map(bone)
	}
}

impl EnemySkillUsage for Enemy {
	fn hold_skill(&self) -> Duration {
		self.enemy_type.hold_skill()
	}

	fn cool_down(&self) -> Duration {
		self.enemy_type.cool_down()
	}

	fn skill_key(&self) -> SlotKey {
		self.enemy_type.skill_key()
	}
}

impl<'w, 's> DerivableFrom<'w, 's, Enemy> for GroundOffset {
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::IfNew;

	type TParam = ();

	fn derive_from(Enemy { enemy_type, .. }: &Enemy, _: &()) -> Self {
		Self::from(enemy_type)
	}
}

impl<TInteractions> Prefab<TInteractions> for Enemy
where
	TInteractions: HandlesAllEffects,
{
	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		assets: &mut impl LoadAsset,
	) -> Result<(), Error> {
		Prefab::<TInteractions>::insert_prefab_components(&self.enemy_type, entity, assets)
	}
}
