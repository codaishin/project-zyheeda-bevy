pub mod queue;
pub mod skill;

use crate::components::Animate;

pub trait GetAnimation<TAnimationKey: Clone + Copy> {
	fn animate(&self) -> Animate<TAnimationKey>;
}

pub trait HasIdle<TAnimationKey: Clone + Copy> {
	const IDLE: Animate<TAnimationKey>;
}
