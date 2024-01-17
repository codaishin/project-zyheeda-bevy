pub mod player;
pub mod projectile;

use crate::components::{Animate, UnitsPerSecond};

pub trait MovementData<TAnimationKey: Clone + Copy> {
	fn get_movement_data(&self) -> (UnitsPerSecond, Animate<TAnimationKey>);
}
