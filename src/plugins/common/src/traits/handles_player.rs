use super::accessors::get::{Getter, GetterRefOptional};
use crate::tools::{movement_animation::MovementAnimation, speed::Speed};
use bevy::prelude::Component;

pub trait HandlesPlayerMovement {
	type TPlayerMovement: Component + Getter<Speed> + GetterRefOptional<MovementAnimation>;
}
