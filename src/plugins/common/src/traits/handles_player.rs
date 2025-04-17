use super::{
	accessors::get::{Getter, GetterRefOptional},
	intersect_at::IntersectAt,
};
use crate::tools::{
	collider_info::ColliderInfo,
	collider_radius::ColliderRadius,
	keys::{movement::MovementKey, slot::SlotKey},
	movement_animation::MovementAnimation,
	speed::Speed,
};
use bevy::{
	math::{InvalidDirectionError, Ray3d},
	prelude::*,
};

pub trait HandlesPlayer {
	type TPlayer: Component;
}

pub trait PlayerMainCamera {
	type TPlayerMainCamera: Component + Default + KeyDirection<MovementKey>;
}

pub trait HandlesPlayerCameras {
	type TCamRay: Resource + GetterRefOptional<Ray3d> + IntersectAt;
}

pub trait HandlesPlayerMouse {
	type TMouseHover: Resource + GetterRefOptional<ColliderInfo<Entity>>;
}

pub trait ConfiguresPlayerMovement {
	type TPlayerMovement: Component
		+ Getter<Speed>
		+ Getter<ColliderRadius>
		+ GetterRefOptional<MovementAnimation>;
}

pub trait ConfiguresPlayerSkillAnimations {
	type TAnimationMarker: Component;

	fn start_skill_animation(slot_key: SlotKey) -> Self::TAnimationMarker;
	fn stop_skill_animation() -> Self::TAnimationMarker;
}

pub trait KeyDirection<TKey> {
	fn key_direction(
		self_transform: &GlobalTransform,
		movement_key: &TKey,
	) -> Result<Dir3, InvalidDirectionError>;
}
