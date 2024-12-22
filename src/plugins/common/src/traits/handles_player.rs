use super::accessors::get::{Getter, GetterRefOptional};
use crate::tools::{movement_animation::MovementAnimation, speed::Speed};
use bevy::prelude::Component;

pub trait HandlesPlayer {
	type TPlayer: Component;
}

pub trait ConfiguresPlayerMovement {
	type TPlayerMovement: Component + Getter<Speed> + GetterRefOptional<MovementAnimation>;
}
