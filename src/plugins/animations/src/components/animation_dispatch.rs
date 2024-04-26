use crate::{
	animation::Animation,
	traits::{HighestPriorityAnimation, InsertAnimation, Priority, RemoveAnimation},
};
use bevy::ecs::component::Component;

#[derive(Component)]
pub struct AnimationDispatch<TAnimation = Animation>(
	Option<TAnimation>,
	Option<TAnimation>,
	Option<TAnimation>,
);

impl<TAnimation> AnimationDispatch<TAnimation> {
	fn slot(&mut self, priority: Priority) -> &mut Option<TAnimation> {
		match priority {
			Priority::High => &mut self.0,
			Priority::Middle => &mut self.1,
			Priority::Low => &mut self.2,
		}
	}
}

impl<TAnimation> Default for AnimationDispatch<TAnimation> {
	fn default() -> Self {
		Self(None, None, None)
	}
}

impl<TAnimation> HighestPriorityAnimation<TAnimation> for AnimationDispatch<TAnimation> {
	fn highest_priority_animation(&self) -> Option<&TAnimation> {
		match self {
			AnimationDispatch(Some(animation), ..) => Some(animation),
			AnimationDispatch(None, Some(animation), ..) => Some(animation),
			AnimationDispatch(None, None, Some(animation)) => Some(animation),
			_ => None,
		}
	}
}

impl<TAnimation> InsertAnimation<TAnimation> for AnimationDispatch<TAnimation> {
	fn insert(&mut self, animation: TAnimation, priority: Priority) {
		let slot = self.slot(priority);

		*slot = Some(animation);
	}
}

impl<TAnimation: PartialEq> RemoveAnimation<TAnimation> for AnimationDispatch<TAnimation> {
	fn remove(&mut self, animation: TAnimation, priority: Priority) {
		let slot = self.slot(priority);

		if slot != &Some(animation) {
			return;
		};

		*slot = None;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug, PartialEq)]
	struct _Animation(&'static str);

	#[test]
	fn insert_low_priority() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation("low"), Priority::Low);

		assert_eq!(
			Some(&_Animation("low")),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn insert_medium_priority() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation("middle"), Priority::Middle);
		dispatch.insert(_Animation("low"), Priority::Low);

		assert_eq!(
			Some(&_Animation("middle")),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn insert_high_priority() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation("high"), Priority::High);
		dispatch.insert(_Animation("middle"), Priority::Middle);

		assert_eq!(
			Some(&_Animation("high")),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn remove_low() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation("low"), Priority::Low);
		dispatch.remove(_Animation("low"), Priority::Low);

		assert_eq!(None, dispatch.highest_priority_animation());
	}

	#[test]
	fn do_not_remove_low_when_animation_mismatch() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation("low"), Priority::Low);
		dispatch.remove(_Animation("other"), Priority::Low);

		assert_eq!(
			Some(&_Animation("low")),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn remove_middle() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation("middle"), Priority::Middle);
		dispatch.remove(_Animation("middle"), Priority::Middle);

		assert_eq!(None, dispatch.highest_priority_animation());
	}

	#[test]
	fn do_not_remove_middle_when_animation_mismatch() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation("middle"), Priority::Middle);
		dispatch.remove(_Animation("other"), Priority::Middle);

		assert_eq!(
			Some(&_Animation("middle")),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn remove_high() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation("high"), Priority::High);
		dispatch.remove(_Animation("high"), Priority::High);

		assert_eq!(None, dispatch.highest_priority_animation());
	}

	#[test]
	fn do_not_remove_high_when_animation_mismatch() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation("high"), Priority::High);
		dispatch.remove(_Animation("other"), Priority::High);

		assert_eq!(
			Some(&_Animation("high")),
			dispatch.highest_priority_animation()
		);
	}
}
