use bevy::{
	prelude::Bundle,
	transform::{components::Transform, TransformBundle},
};
use bevy_rapier3d::{
	geometry::{ActiveEvents, Collider},
	prelude::ActiveCollisionTypes,
};

#[derive(Bundle, Clone, Default)]
pub struct ColliderBundle {
	pub collider: Collider,
	pub active_events: ActiveEvents,
	pub active_collision_types: ActiveCollisionTypes,
}

impl ColliderBundle {
	pub fn new_static_collider(collider: Collider) -> Self {
		Self {
			collider,
			active_events: ActiveEvents::COLLISION_EVENTS,
			active_collision_types: ActiveCollisionTypes::STATIC_STATIC,
		}
	}
}

#[derive(Bundle, Clone, Default)]
pub struct ColliderTransformBundle {
	pub collider: Collider,
	pub transform: TransformBundle,
	pub active_events: ActiveEvents,
	pub active_collision_types: ActiveCollisionTypes,
}

impl ColliderTransformBundle {
	pub fn new_static_collider(transform: Transform, collider: Collider) -> Self {
		Self {
			transform: TransformBundle::from_transform(transform),
			collider,
			active_events: ActiveEvents::COLLISION_EVENTS,
			active_collision_types: ActiveCollisionTypes::STATIC_STATIC,
		}
	}
}
