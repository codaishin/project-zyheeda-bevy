use crate::components::{BoneName, Equipment, Item, Queue, SlotBones, SlotKey, Slots};
use bevy::ecs::bundle::Bundle;
use common::components::{Collection, Idle};

#[derive(Bundle)]
pub struct Loadout {
	slot_bones: SlotBones,
	slots: Slots,
	equipment: Equipment,
	dequeue_next: Idle,
	queue: Queue,
}

impl Loadout {
	pub fn new<const B: usize, const E: usize>(
		slot_bones: [(SlotKey, &'static BoneName); B],
		equipment: [(SlotKey, Option<Item>); E],
	) -> Self {
		Self {
			slot_bones: SlotBones(slot_bones.into()),
			equipment: Collection(equipment.into()),
			slots: Slots::new(),
			dequeue_next: Idle,
			queue: Queue([].into()),
		}
	}
}
