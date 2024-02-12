pub mod player;
pub mod projectile;
pub mod void_sphere;

use crate::components::Animate;
use common::tools::UnitsPerSecond;

pub trait MovementData<TAnimationKey: Clone + Copy> {
	fn get_movement_data(&self) -> (UnitsPerSecond, Animate<TAnimationKey>);
}
