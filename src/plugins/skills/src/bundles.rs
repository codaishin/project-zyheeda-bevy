use crate::{
	components::{queue::Queue, slots::Slots, BoneName, Equipment, Mounts, SkillSpawn, SlotBones},
	items::{slot_key::SlotKey, Item},
};
use bevy::ecs::bundle::Bundle;
use common::components::{Collection, Idle};

#[derive(Bundle)]
pub struct Loadout {
	skill_spawn: SkillSpawn<&'static BoneName>,
	slot_bones: SlotBones,
	slots: Slots,
	equipment: Equipment,
	dequeue_next: Idle,
	queue: Queue,
}

impl Loadout {
	pub fn new<const B: usize, const E: usize>(
		skill_spawn: &'static str,
		slot_bones: [(SlotKey, Mounts<&'static BoneName>); B],
		equipment: [(SlotKey, Option<Item>); E],
	) -> Self {
		Self {
			skill_spawn: SkillSpawn(skill_spawn),
			slot_bones: SlotBones(slot_bones.into()),
			equipment: Collection(equipment.into()),
			slots: Slots::new(),
			dequeue_next: Idle,
			queue: Queue::default(),
		}
	}
}
