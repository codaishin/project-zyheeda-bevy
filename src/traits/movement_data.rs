use crate::{behaviors::MovementMode, components::UnitsPerSecond};
pub mod player;

pub trait MovementData {
	fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode);
}
