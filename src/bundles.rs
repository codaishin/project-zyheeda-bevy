use crate::components::{Equip, Queue, SlotBones, Slots, WaitNext};
use bevy::prelude::Bundle;

#[derive(Bundle)]
pub struct Loadout<TBehaviorItem, TBehaviorAgent>
where
	TBehaviorItem: Sync + Send + Copy + 'static,
	TBehaviorAgent: Sync + Send + Copy + 'static,
{
	slot_bones: SlotBones,
	slots: Slots<TBehaviorItem>,
	equipment: Equip<TBehaviorItem>,
	wait_next: WaitNext<TBehaviorAgent>,
	queue: Queue<TBehaviorAgent>,
}

impl<TBehaviorItem, TBehavior> Loadout<TBehaviorItem, TBehavior>
where
	TBehaviorItem: Sync + Send + Copy + 'static,
	TBehavior: Sync + Send + Copy + 'static,
{
	pub fn new(slot_bones: SlotBones, equipment: Equip<TBehaviorItem>) -> Self {
		Self {
			slot_bones,
			equipment,
			slots: Slots::new(),
			wait_next: WaitNext::new(),
			queue: Queue([].into()),
		}
	}
}
