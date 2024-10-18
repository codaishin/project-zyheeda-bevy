use crate::{
	components::{
		lookup::Lookup,
		queue::Queue,
		skill_spawners::SkillSpawners,
		SlotDefinition,
		SlotsDefinition,
	},
	definitions::{
		item_slots::{ForearmSlots, HandSlots},
		sub_models::SubModels,
	},
};
use bevy::{ecs::bundle::Bundle, prelude::default};
use common::components::Idle;

#[derive(Bundle)]
pub struct Loadout<TAgent>
where
	TAgent: Sync + Send + 'static,
{
	sub_models: Lookup<SubModels<TAgent>>,
	hand_slots: Lookup<HandSlots<TAgent>>,
	forearm_slots: Lookup<ForearmSlots<TAgent>>,
	skill_spawners: SkillSpawners,
	slot_definition: SlotsDefinition,
	idle: Idle,
	queue: Queue,
}

impl<TAgent> Loadout<TAgent>
where
	TAgent: Sync + Send + 'static,
{
	pub fn new<const N: usize>(slots_definitions: [SlotDefinition; N]) -> Self {
		Self {
			sub_models: default(),
			hand_slots: default(),
			forearm_slots: default(),
			skill_spawners: default(),
			slot_definition: SlotsDefinition::new(slots_definitions),
			idle: Idle,
			queue: default(),
		}
	}
}
