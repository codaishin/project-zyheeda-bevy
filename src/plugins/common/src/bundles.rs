use bevy::{
	prelude::{Bundle, GlobalTransform, InheritedVisibility, ViewVisibility, Visibility},
	transform::components::Transform,
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

#[derive(Bundle, Clone, Default)]
pub struct ColliderTransformBundle {
	pub collider: Collider,
	pub transform: Transform,
	pub global_transform: GlobalTransform,
	pub active_events: ActiveEvents,
	pub active_collision_types: ActiveCollisionTypes,
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
