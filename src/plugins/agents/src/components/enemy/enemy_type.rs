use crate::components::enemy::{Enemy, void_sphere::VoidSphere};
use bevy::asset::AssetPath;
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		handles_enemies::{EnemySkillUsage, EnemyType},
		loadout::LoadoutConfig,
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
