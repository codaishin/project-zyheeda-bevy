use crate::components::{Equip, Queue, SlotBones, Slots, WaitNext};
use bevy::prelude::Bundle;

#[derive(Bundle)]
pub struct Loadout {
	slot_bones: SlotBones,
	slots: Slots,
	equipment: Equip,
	wait_next: WaitNext,
	queue: Queue,
}

impl Loadout {
	pub fn new(slot_bones: SlotBones, equipment: Equip) -> Self {
		Self {
			slot_bones,
			equipment,
			slots: Slots::new(),
			wait_next: WaitNext,
			queue: Queue([].into()),
		}
	}
}
