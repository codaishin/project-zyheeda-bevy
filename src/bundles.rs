use crate::{
	components::{Collection, DequeueNext, Equipment, Item, Queue, SlotBones, SlotKey, Slots},
	types::BoneName,
};
use bevy::{
	prelude::Bundle,
	transform::{components::Transform, TransformBundle},
};
use bevy_rapier3d::{
	geometry::{ActiveEvents, Collider, Sensor},
	prelude::ActiveCollisionTypes,
};

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

#[derive(Bundle, Clone, Default)]
pub struct ColliderBundle<TExtra: Bundle + Clone + Default> {
	pub collider: Collider,
	pub transform: TransformBundle,
	pub active_events: ActiveEvents,
	pub active_collision_types: ActiveCollisionTypes,
	pub extra: TExtra,
}

impl ColliderBundle<Sensor> {
	pub fn new_static_collider(transform: Transform, collider: Collider) -> Self {
		Self {
			transform: TransformBundle::from_transform(transform),
			collider,
			active_events: ActiveEvents::COLLISION_EVENTS,
			active_collision_types: ActiveCollisionTypes::STATIC_STATIC,
			extra: Sensor,
		}
	}
}
