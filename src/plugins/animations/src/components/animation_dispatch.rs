use crate::{
	animation::Animation,
	traits::{
		AnimationChainUpdate,
		HighestPriorityAnimation,
		InsertAnimation,
		Priority,
		RemoveAnimation,
	},
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

impl<TAnimation: AnimationChainUpdate> InsertAnimation<TAnimation>
	for AnimationDispatch<TAnimation>
{
	fn insert(&mut self, mut animation: TAnimation, priority: Priority) {
		let slot = self.slot(priority);

		if let Some(last) = slot {
			animation.chain_update(last);
		}

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
	use bevy::prelude::default;

	#[derive(Debug, PartialEq, Clone)]
	struct _Animation {
		name: &'static str,
		chain_update_calls: Vec<_Animation>,
	}

	impl _Animation {
		fn new(name: &'static str) -> Self {
			Self {
				name,
				chain_update_calls: default(),
			}
		}
	}

	impl AnimationChainUpdate for _Animation {
		fn chain_update(&mut self, last: &Self) {
			self.chain_update_calls.push(last.clone())
		}
	}

	#[test]
	fn insert_low_priority() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("low"), Priority::Low);

		assert_eq!(
			Some(&_Animation::new("low")),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn insert_medium_priority() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("middle"), Priority::Middle);
		dispatch.insert(_Animation::new("low"), Priority::Low);

		assert_eq!(
			Some(&_Animation::new("middle")),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn insert_high_priority() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("high"), Priority::High);
		dispatch.insert(_Animation::new("middle"), Priority::Middle);

		assert_eq!(
			Some(&_Animation::new("high")),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn remove_low() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("low"), Priority::Low);
		dispatch.remove(_Animation::new("low"), Priority::Low);

		assert_eq!(None, dispatch.highest_priority_animation());
	}

	#[test]
	fn do_not_remove_low_when_animation_mismatch() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("low"), Priority::Low);
		dispatch.remove(_Animation::new("other"), Priority::Low);

		assert_eq!(
			Some(&_Animation::new("low")),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn remove_middle() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("middle"), Priority::Middle);
		dispatch.remove(_Animation::new("middle"), Priority::Middle);

		assert_eq!(None, dispatch.highest_priority_animation());
	}

	#[test]
	fn do_not_remove_middle_when_animation_mismatch() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("middle"), Priority::Middle);
		dispatch.remove(_Animation::new("other"), Priority::Middle);

		assert_eq!(
			Some(&_Animation::new("middle")),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn remove_high() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("high"), Priority::High);
		dispatch.remove(_Animation::new("high"), Priority::High);

		assert_eq!(None, dispatch.highest_priority_animation());
	}

	#[test]
	fn do_not_remove_high_when_animation_mismatch() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("high"), Priority::High);
		dispatch.remove(_Animation::new("other"), Priority::High);

		assert_eq!(
			Some(&_Animation::new("high")),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn call_chain_update() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("last"), Priority::High);
		dispatch.insert(_Animation::new("mock"), Priority::High);

		let mock = dispatch.highest_priority_animation().unwrap();

		assert_eq!(vec![_Animation::new("last")], mock.chain_update_calls);
	}
}
