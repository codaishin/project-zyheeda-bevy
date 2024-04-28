use crate::components::MovementMode;
use animations::animation::Animation;
use bevy::ecs::component::Component;
use common::traits::get::Get;

#[derive(Component)]
pub struct MovementAnimations<TAnimation = Animation> {
	slow: TAnimation,
	fast: TAnimation,
}

impl MovementAnimations {
	pub fn new(fast: Animation, slow: Animation) -> Self {
		Self { slow, fast }
	}
}

impl<TAnimation> Get<MovementMode, TAnimation> for MovementAnimations<TAnimation> {
	fn get(&self, key: &MovementMode) -> &TAnimation {
		match key {
			MovementMode::Fast => &self.fast,
			MovementMode::Slow => &self.slow,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Default, Debug, PartialEq)]
	struct _Animation(&'static str);

	#[test]
	fn get_fast() {
		let animation = MovementAnimations {
			slow: _Animation::default(),
			fast: _Animation("fast"),
		};

		assert_eq!(&_Animation("fast"), animation.get(&MovementMode::Fast));
	}

	#[test]
	fn get_slow() {
		let animation = MovementAnimations {
			slow: _Animation("slow"),
			fast: _Animation::default(),
		};

		assert_eq!(&_Animation("slow"), animation.get(&MovementMode::Slow));
	}
}
