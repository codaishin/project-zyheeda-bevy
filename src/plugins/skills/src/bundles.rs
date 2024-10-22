use crate::{
	components::{
		combo_node::ComboNode,
		combos_time_out::CombosTimeOut,
		queue::Queue,
		skill_executer::SkillExecuter,
		skill_spawners::SkillSpawners,
		slots::Slots,
	},
	definitions::{
		item_slots::{ForearmSlots, HandSlots},
		sub_models::SubModels,
	},
	item::SkillItem,
	skills::{RunSkillBehavior, SkillId},
	slot_key::SlotKey,
};
use bevy::{ecs::bundle::Bundle, prelude::default};
use common::components::Idle;
use items::components::visualizer::Visualizer;
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
pub struct Loadout<TAgent>
where
	TAgent: Sync + Send + 'static,
{
	slot_definition: Slots<SkillId>,
	skill_execution: ExecutionBundle,
	item_visualization: ItemVisualizationBundle<TAgent>,
}

impl<TAgent> Loadout<TAgent>
where
	TAgent: Sync + Send + 'static,
{
	pub fn new<const N: usize>(
		slots_definitions: [(SlotKey, Option<SkillItem<SkillId>>); N],
	) -> Self {
		Self {
			slot_definition: Slots(HashMap::from(slots_definitions)),
			skill_execution: default(),
			item_visualization: default(),
		}
	}
}

#[derive(Bundle)]
struct ItemVisualizationBundle<TAgent>
where
	TAgent: Sync + Send + 'static,
{
	sub_models: Visualizer<SubModels<TAgent>>,
	hand_slots: Visualizer<HandSlots<TAgent>>,
	forearm_slots: Visualizer<ForearmSlots<TAgent>>,
}

impl<TAgent> Default for ItemVisualizationBundle<TAgent>
where
	TAgent: Sync + Send + 'static,
{
	fn default() -> Self {
		Self {
			sub_models: default(),
			hand_slots: default(),
			forearm_slots: default(),
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
