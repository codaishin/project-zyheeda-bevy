use super::{
	accessors::get::{Getter, GetterRefOptional},
	intersect_at::IntersectAt,
};
use crate::tools::{
	collider_info::ColliderInfo,
	movement_animation::MovementAnimation,
	slot_key::SlotKey,
	speed::Speed,
};
use bevy::{math::Ray3d, prelude::*};

pub trait HandlesPlayer {
	type TPlayer: Component;
}

pub trait WithMainCamera {
	type TWithMainCam<TMainCamera>;

	fn with_main_camera<TMainCamera>(self) -> Self::TWithMainCam<TMainCamera>
	where
		TMainCamera: Component;
}

pub trait HandlesPlayerCameras {
	type TCamRay: Resource + GetterRefOptional<Ray3d> + IntersectAt;
}

pub trait HandlesPlayerMouse {
	type TMouseHover: Resource + GetterRefOptional<ColliderInfo<Entity>>;
}

pub trait ConfiguresPlayerMovement {
	type TPlayerMovement: Component + Getter<Speed> + GetterRefOptional<MovementAnimation>;
}

pub trait ConfiguresPlayerSkillAnimations {
	type TAnimationMarker: Component;

	fn start_skill_animation(slot_key: SlotKey) -> Self::TAnimationMarker;
	fn stop_skill_animation() -> Self::TAnimationMarker;
}
