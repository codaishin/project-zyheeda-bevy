use crate::{components::MovementMode, traits::GetAnimation};
use bevy::ecs::component::Component;
use common::traits::animation::Animation;

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

impl GetAnimation for MovementAnimations {
	fn animation(&self, key: &MovementMode) -> &Animation {
		match key {
			MovementMode::Fast => &self.fast,
			MovementMode::Slow => &self.slow,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::{animation::PlayMode, load_asset::Path};

	#[derive(Default, Debug, PartialEq)]
	struct _Animation(&'static str);

	#[test]
	fn get_fast() {
		let animation = MovementAnimations {
			slow: Animation::new(Path::from("slow"), PlayMode::Repeat),
			fast: Animation::new(Path::from("fast"), PlayMode::Repeat),
		};

		assert_eq!(
			&Animation::new(Path::from("fast"), PlayMode::Repeat),
			animation.animation(&MovementMode::Fast)
		);
	}

	#[test]
	fn get_slow() {
		let animation = MovementAnimations {
			slow: Animation::new(Path::from("slow"), PlayMode::Repeat),
			fast: Animation::new(Path::from("fast"), PlayMode::Repeat),
		};

		assert_eq!(
			&Animation::new(Path::from("slow"), PlayMode::Repeat),
			animation.animation(&MovementMode::Slow)
		);
	}
}
