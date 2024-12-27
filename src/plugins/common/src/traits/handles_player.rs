use super::accessors::get::{Getter, GetterRefOptional};
use crate::tools::{movement_animation::MovementAnimation, slot_key::SlotKey, speed::Speed};
use bevy::prelude::Component;

pub trait HandlesPlayer {
	type TPlayer: Component;
}

pub trait ConfiguresPlayerMovement {
	type TPlayerMovement: Component + Getter<Speed> + GetterRefOptional<MovementAnimation>;
}

pub trait ConfiguresPlayerSkillAnimations {
	type TAnimationMarker: Component;

	fn start_skill_animation(slot_key: SlotKey) -> Self::TAnimationMarker;
	fn stop_skill_animation() -> Self::TAnimationMarker;
}
