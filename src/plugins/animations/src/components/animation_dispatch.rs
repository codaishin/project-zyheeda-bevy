use crate::{
	animation::Animation,
	traits::{
		AnimationChainUpdate,
		HighestPriorityAnimation,
		InsertAnimation,
		MarkObsolete,
		Priority,
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

impl<TAnimation> MarkObsolete<TAnimation> for AnimationDispatch<TAnimation> {
	fn mark_obsolete(&mut self, priority: Priority) {
		let slot = self.slot(priority);

		*slot = None;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{prelude::default, utils::Uuid};

	#[derive(Default, Debug, PartialEq, Clone)]
	struct _Animation {
		uuid: Uuid,
		name: Option<&'static str>,
		chain_update_calls: Vec<_Animation>,
	}

	impl _Animation {
		fn new(uuid: Uuid) -> Self {
			Self {
				uuid,
				name: None,
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
		let uuid = Uuid::new_v4();
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new(uuid), Priority::Low);

		assert_eq!(
			Some(&_Animation::new(uuid)),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn insert_medium_priority() {
		let uuid_middle = Uuid::new_v4();
		let uuid_low = Uuid::new_v4();
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new(uuid_middle), Priority::Middle);
		dispatch.insert(_Animation::new(uuid_low), Priority::Low);

		assert_eq!(
			Some(&_Animation::new(uuid_middle)),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn insert_high_priority() {
		let uuid_high = Uuid::new_v4();
		let uuid_middle = Uuid::new_v4();
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new(uuid_high), Priority::High);
		dispatch.insert(_Animation::new(uuid_middle), Priority::Middle);

		assert_eq!(
			Some(&_Animation::new(uuid_high)),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn mark_obsolete_low() {
		let uuid = Uuid::new_v4();
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new(uuid), Priority::Low);
		dispatch.mark_obsolete(Priority::Low);

		assert_eq!(None, dispatch.highest_priority_animation());
	}

	#[test]
	fn mark_obsolete_middle() {
		let uuid = Uuid::new_v4();
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new(uuid), Priority::Middle);
		dispatch.mark_obsolete(Priority::Middle);

		assert_eq!(None, dispatch.highest_priority_animation());
	}

	#[test]
	fn mark_obsolete_high() {
		let uuid = Uuid::new_v4();
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new(uuid), Priority::High);
		dispatch.mark_obsolete(Priority::High);

		assert_eq!(None, dispatch.highest_priority_animation());
	}

	#[test]
	fn call_chain_update() {
		let uuid_last = Uuid::new_v4();
		let uuid_mock = Uuid::new_v4();
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new(uuid_last), Priority::High);
		dispatch.insert(_Animation::new(uuid_mock), Priority::High);

		let mock = dispatch.highest_priority_animation().unwrap();

		assert_eq!(vec![_Animation::new(uuid_last)], mock.chain_update_calls);
	}
}
