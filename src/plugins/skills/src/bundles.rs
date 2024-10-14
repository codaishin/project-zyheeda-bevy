use crate::components::{
	queue::Queue,
	skill_spawners::SkillSpawners,
	SlotDefinition,
	SlotsDefinition,
};
use bevy::{ecs::bundle::Bundle, prelude::default};
use common::components::Idle;

#[derive(Bundle)]
pub struct Loadout {
	skill_spawners: SkillSpawners,
	slot_definition: SlotsDefinition,
	dequeue_next: Idle,
	queue: Queue,
}

impl Loadout {
	pub fn new<const N: usize>(slots_definitions: [SlotDefinition; N]) -> Self {
		Self {
			skill_spawners: SkillSpawners::default(),
			slot_definition: SlotsDefinition::new(slots_definitions),
			dequeue_next: Idle,
			queue: default(),
		}
	}
}
