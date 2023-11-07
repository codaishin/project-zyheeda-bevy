use crate::components::{Equip, Queue, SlotBones, Slots, WaitNext};
use bevy::prelude::Bundle;

#[derive(Bundle)]
pub struct Loadout<TBehavior>
where
	TBehavior: Sync + Send + Copy + 'static,
{
	slot_bones: SlotBones,
	slots: Slots<TBehavior>,
	equipment: Equip<TBehavior>,
	wait_next: WaitNext<TBehavior>,
	queue: Queue<TBehavior>,
}

impl<TBehavior> Loadout<TBehavior>
where
	TBehavior: Sync + Send + Copy + 'static,
{
	pub fn new(slot_bones: SlotBones, equipment: Equip<TBehavior>) -> Self {
		Self {
			slot_bones,
			equipment,
			slots: Slots::new(),
			wait_next: WaitNext::new(),
			queue: Queue([].into()),
		}
	}
}
