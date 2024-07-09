use crate::components::{queue::Queue, BoneName, SkillSpawn, SlotDefinition, SlotsDefinition};
use bevy::{ecs::bundle::Bundle, prelude::default};
use common::components::Idle;

#[derive(Bundle)]
pub struct Loadout {
	skill_spawn: SkillSpawn<&'static BoneName>,
	slot_definition: SlotsDefinition,
	dequeue_next: Idle,
	queue: Queue,
}

impl Loadout {
	pub fn new<const N: usize>(
		skill_spawn: &'static str,
		slots_definitions: [SlotDefinition; N],
	) -> Self {
		Self {
			skill_spawn: SkillSpawn(skill_spawn),
			slot_definition: SlotsDefinition::new(slots_definitions),
			dequeue_next: Idle,
			queue: default(),
		}
	}
}
