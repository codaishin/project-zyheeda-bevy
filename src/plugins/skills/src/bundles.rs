use crate::{
	components::{queue::Queue, slots::Slots, BoneName, Mounts, SkillSpawn, SlotsDefinition},
	items::{slot_key::SlotKey, Item},
};
use bevy::{ecs::bundle::Bundle, prelude::default};
use common::{components::Idle, traits::load_asset::Path};

#[derive(Bundle)]
pub struct Loadout {
	skill_spawn: SkillSpawn<&'static BoneName>,
	slot_definition: SlotsDefinition,
	slots: Slots<Path>,
	dequeue_next: Idle,
	queue: Queue,
}

type SlotDefinition = (SlotKey, (Mounts<&'static BoneName>, Option<Item<Path>>));

impl Loadout {
	pub fn new<const N: usize>(
		skill_spawn: &'static str,
		slots_definitions: [SlotDefinition; N],
	) -> Self {
		Self {
			skill_spawn: SkillSpawn(skill_spawn),
			slot_definition: SlotsDefinition(slots_definitions.into()),
			slots: default(),
			dequeue_next: Idle,
			queue: default(),
		}
	}
}
