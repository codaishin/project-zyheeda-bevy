pub(crate) mod enemy_type;
pub(crate) mod void_sphere;

use crate::components::{agent::Agent, enemy::enemy_type::EnemyTypeInternal};
use bevy::{asset::AssetPath, prelude::*};
use bevy_rapier3d::prelude::{GravityScale, RigidBody};
use common::{
	attributes::{effect_target::EffectTarget, health::Health},
	components::{
		collider_relationship::InteractionTarget,
		ground_offset::GroundOffset,
		is_blocker::{Blocker, IsBlocker},
		persistent_entity::PersistentEntity,
	},
	effects::{force::Force, gravity::Gravity},
	errors::Error,
	tools::{
		Units,
		UnitsPerSecond,
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
		accessors::get::GetProperty,
		animation::Animation,
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
	Agent,
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

impl GetProperty<Speed> for Enemy {
	fn get_property(&self) -> UnitsPerSecond {
		self.speed.0
	}
}

impl GetProperty<Option<MovementAnimation>> for Enemy {
	fn get_property(&self) -> Option<&'_ Animation> {
		self.movement_animation
			.as_ref()
			.map(|MovementAnimation(animation)| animation)
	}
}

impl GetProperty<AggroRange> for Enemy {
	fn get_property(&self) -> Units {
		self.aggro_range.0
	}
}

impl GetProperty<AttackRange> for Enemy {
	fn get_property(&self) -> Units {
		self.attack_range.0
	}
}

impl GetProperty<EnemyTarget> for Enemy {
	fn get_property(&self) -> EnemyTarget {
		self.target
	}
}

impl GetProperty<ColliderRadius> for Enemy {
	fn get_property(&self) -> Units {
		self.collider_radius.0
	}
}

impl GetProperty<AttributeOnSpawn<Health>> for Enemy {
	fn get_property(&self) -> Health {
		match self.enemy_type {
			EnemyTypeInternal::VoidSphere(_) => Health::new(5.),
		}
	}
}

impl GetProperty<AttributeOnSpawn<EffectTarget<Gravity>>> for Enemy {
	fn get_property(&self) -> EffectTarget<Gravity> {
		match self.enemy_type {
			EnemyTypeInternal::VoidSphere(_) => EffectTarget::Affected,
		}
	}
}

impl GetProperty<AttributeOnSpawn<EffectTarget<Force>>> for Enemy {
	fn get_property(&self) -> EffectTarget<Force> {
		match self.enemy_type {
			EnemyTypeInternal::VoidSphere(_) => EffectTarget::Affected,
		}
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

	fn derive_from(_: Entity, Enemy { enemy_type, .. }: &Enemy, _: &()) -> Self {
		Self::from(enemy_type)
	}
}

impl Prefab<()> for Enemy {
	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		assets: &mut impl LoadAsset,
	) -> Result<(), Error> {
		Prefab::<()>::insert_prefab_components(&self.enemy_type, entity, assets)
	}
}
