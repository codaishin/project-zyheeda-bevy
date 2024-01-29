use crate::{
	components::{Collection, DequeueNext, Equipment, Item, Queue, SlotBones, SlotKey, Slots},
	types::BoneName,
};
use bevy::prelude::Bundle;

#[derive(Bundle)]
pub struct Loadout {
	slot_bones: SlotBones,
	slots: Slots,
	equipment: Equipment,
	dequeue_next: DequeueNext,
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
			dequeue_next: DequeueNext,
			queue: Queue([].into()),
		}
	}
}
