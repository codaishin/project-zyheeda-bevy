use super::{
	accessors::get::{Getter, GetterRefOptional},
	animation::Animation,
};
use crate::tools::{movement_animation::MovementAnimation, speed::Speed};
use bevy::prelude::Component;

pub trait HandlesPlayer {
	type TPlayer: Component + PlayerAnimations;
}

pub trait ConfiguresPlayerMovement {
	type TPlayerMovement: Component + Getter<Speed> + GetterRefOptional<MovementAnimation>;
}

pub trait PlayerAnimations {
	fn animation(slot_key: SlotKey) -> Animation;
}
