use bevy::{
	prelude::{Bundle, GlobalTransform, InheritedVisibility, ViewVisibility, Visibility},
	transform::{bundles::TransformBundle, components::Transform},
};
use bevy_rapier3d::{
	geometry::{ActiveEvents, Collider},
	prelude::ActiveCollisionTypes,
};

use crate::components::AssetModel;

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

#[derive(Bundle, Default)]
pub struct AssetModelBundle {
	pub model: AssetModel,
	pub visibility: Visibility,
	pub inherited_visibility: InheritedVisibility,
	pub view_visibility: ViewVisibility,
	pub transform: Transform,
	pub global_transform: GlobalTransform,
}
