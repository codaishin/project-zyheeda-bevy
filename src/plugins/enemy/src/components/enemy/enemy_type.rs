use crate::components::enemy::{Enemy, void_sphere::VoidSphere};
use bevy::asset::AssetPath;
use common::{
	components::ground_offset::GroundOffset,
	errors::Error,
	tools::{action_key::slot::SlotKey, bone::Bone},
	traits::{
		handles_enemies::{EnemySkillUsage, EnemyType},
		handles_skill_behaviors::SkillSpawner,
		load_asset::LoadAsset,
		loadout::LoadoutConfig,
		mapper::Mapper,
		prefab::{Prefab, PrefabEntityCommands},
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot, VisibleSlots},
	},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) enum EnemyTypeInternal {
	VoidSphere(VoidSphere),
}

macro_rules! use_matched {
	($v:expr, $f:expr) => {
		match $v {
			EnemyTypeInternal::VoidSphere(e) => $f(e),
		}
	};
}

impl EnemyTypeInternal {
	pub(crate) fn new_enemy(enemy_type: EnemyType) -> Enemy {
		match enemy_type {
			EnemyType::VoidSphere => VoidSphere::new_enemy(),
		}
	}
}

impl LoadoutConfig for EnemyTypeInternal {
	fn inventory(&self) -> impl Iterator<Item = Option<AssetPath<'static>>> {
		use_matched!(self, LoadoutConfig::inventory)
	}

	fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<AssetPath<'static>>)> {
		use_matched!(self, LoadoutConfig::slots)
	}
}

impl EnemySkillUsage for EnemyTypeInternal {
	fn hold_skill(&self) -> Duration {
		use_matched!(self, EnemySkillUsage::hold_skill)
	}

	fn cool_down(&self) -> Duration {
		use_matched!(self, EnemySkillUsage::cool_down)
	}

	fn skill_key(&self) -> SlotKey {
		use_matched!(self, EnemySkillUsage::skill_key)
	}
}

impl VisibleSlots for EnemyTypeInternal {
	fn visible_slots(&self) -> impl Iterator<Item = SlotKey> {
		use_matched!(self, VisibleSlots::visible_slots)
	}
}

impl Mapper<Bone<'_>, Option<EssenceSlot>> for EnemyTypeInternal {
	fn map(&self, bone: Bone<'_>) -> Option<EssenceSlot> {
		use_matched!(self, |e| Mapper::map(e, bone))
	}
}

impl Mapper<Bone<'_>, Option<HandSlot>> for EnemyTypeInternal {
	fn map(&self, bone: Bone<'_>) -> Option<HandSlot> {
		use_matched!(self, |e| Mapper::map(e, bone))
	}
}

impl Mapper<Bone<'_>, Option<ForearmSlot>> for EnemyTypeInternal {
	fn map(&self, bone: Bone<'_>) -> Option<ForearmSlot> {
		use_matched!(self, |e| Mapper::map(e, bone))
	}
}

impl Mapper<Bone<'_>, Option<SkillSpawner>> for EnemyTypeInternal {
	fn map(&self, bone: Bone) -> Option<SkillSpawner> {
		use_matched!(self, |e| Mapper::map(e, bone))
	}
}

impl From<&EnemyTypeInternal> for GroundOffset {
	fn from(value: &EnemyTypeInternal) -> Self {
		use_matched!(value, Self::from)
	}
}

impl Prefab<()> for EnemyTypeInternal {
	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		assets: &mut impl LoadAsset,
	) -> Result<(), Error> {
		use_matched!(self, |e| Prefab::<()>::insert_prefab_components(
			e, entity, assets
		))
	}
}
