use crate::{components::MovementMode, traits::GetAnimation};
use animations::animation::Animation;
use bevy::ecs::component::Component;

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

impl<TAnimation> GetAnimation<TAnimation> for MovementAnimations<TAnimation> {
	fn animation(&self, key: &MovementMode) -> &TAnimation {
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

		assert_eq!(
			&_Animation("fast"),
			animation.animation(&MovementMode::Fast)
		);
	}

	#[test]
	fn get_slow() {
		let animation = MovementAnimations {
			slow: _Animation("slow"),
			fast: _Animation::default(),
		};

		assert_eq!(
			&_Animation("slow"),
			animation.animation(&MovementMode::Slow)
		);
	}
}
