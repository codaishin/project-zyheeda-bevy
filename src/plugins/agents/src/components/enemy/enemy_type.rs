use crate::components::enemy::void_sphere::VoidSphere;
use common::{tools::action_key::slot::SlotKey, traits::handles_enemies::EnemySkillUsage};
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
