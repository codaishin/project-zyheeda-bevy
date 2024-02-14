use crate::{components::SlotBones, types::BoneName};
use behaviors::components::Idle;
use bevy::{
	prelude::Bundle,
	transform::{components::Transform, TransformBundle},
};
use bevy_rapier3d::{
	geometry::{ActiveEvents, Collider},
	prelude::ActiveCollisionTypes,
};
use common::components::{Collection, Equipment, Item, Queue, SlotKey, Slots};

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

#[derive(Bundle, Clone, Default)]
pub struct ColliderBundle {
	pub collider: Collider,
	pub transform: TransformBundle,
	pub active_events: ActiveEvents,
	pub active_collision_types: ActiveCollisionTypes,
}

impl ColliderBundle {
	pub fn new_static_collider(transform: Transform, collider: Collider) -> Self {
		Self {
			transform: TransformBundle::from_transform(transform),
			collider,
			active_events: ActiveEvents::COLLISION_EVENTS,
			active_collision_types: ActiveCollisionTypes::STATIC_STATIC,
		}
	}
}
