use crate::{
	components::{
		combo_node::ComboNode,
		combos_time_out::CombosTimeOut,
		queue::Queue,
		skill_executer::SkillExecuter,
		skill_spawners::SkillSpawners,
		slots::Slots,
	},
	item::SkillItem,
	skills::{RunSkillBehavior, SkillId},
	slot_key::SlotKey,
};
use bevy::prelude::*;
use common::components::Idle;
use std::collections::HashMap;

#[derive(Bundle)]
pub struct ComboBundle {
	combos: ComboNode<SkillId>,
	timeout: CombosTimeOut,
}

impl ComboBundle {
	pub fn with_timeout(timeout: CombosTimeOut) -> Self {
		Self {
			combos: default(),
			timeout,
		}
	}

	pub fn with_predefined_combos<const N: usize>(
		mut self,
		combos: [(SlotKey, (SkillId, ComboNode<SkillId>)); N],
	) -> Self {
		self.combos = ComboNode::new(combos);
		self
	}
}

#[derive(Bundle)]
pub struct Loadout {
	slot_definition: Slots<SkillId>,
	skill_execution: ExecutionBundle,
}

impl Loadout {
	pub fn new<const N: usize>(
		slots_definitions: [(SlotKey, Option<SkillItem<SkillId>>); N],
	) -> Self {
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
	skill_spawners: SkillSpawners,
	idle: Idle,
}

impl Default for ExecutionBundle {
	fn default() -> Self {
		Self {
			queue: default(),
			executor: default(),
			skill_spawners: default(),
			idle: Idle,
		}
	}
}
