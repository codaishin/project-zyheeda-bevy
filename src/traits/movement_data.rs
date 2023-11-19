pub mod player;
pub mod projectile;

use crate::{behaviors::MovementMode, components::UnitsPerSecond};

pub trait MovementData {
	fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode);
}
