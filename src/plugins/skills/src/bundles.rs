use crate::{
	components::{
		combos::Combos,
		combos_time_out::CombosTimeOut,
		queue::Queue,
		skill_executer::SkillExecuter,
		slots::Slots,
	},
	item::Item,
	skills::RunSkillBehavior,
};
use bevy::prelude::*;
use common::tools::action_key::slot::SlotKey;
use std::collections::HashMap;

#[derive(Bundle)]
pub struct ComboBundle {
	combos: Combos,
	timeout: CombosTimeOut,
}

impl ComboBundle {
	pub fn with_timeout(timeout: CombosTimeOut) -> Self {
		Self {
			combos: default(),
			timeout,
		}
	}
}

#[derive(Bundle)]
pub struct Loadout {
	slot_definition: Slots,
	skill_execution: ExecutionBundle,
}

impl Loadout {
	pub fn new<const N: usize>(slots_definitions: [(SlotKey, Option<Handle<Item>>); N]) -> Self {
		Self {
			slot_definition: Slots(HashMap::from(slots_definitions)),
			skill_execution: default(),
		}
	}
}

#[derive(Bundle)]
struct ExecutionBundle {
	queue: Queue,
	executor: SkillExecuter<RunSkillBehavior>,
}

impl Default for ExecutionBundle {
	fn default() -> Self {
		Self {
			queue: default(),
			executor: default(),
		}
	}
}
