use crate::{
	animation::Animation,
	traits::{
		AnimationChainUpdate,
		FlushObsolete,
		HighestPriorityAnimation,
		InsertAnimation,
		MarkObsolete,
		Priority,
	},
};
use bevy::ecs::component::Component;
use std::{fmt::Debug, mem};

#[derive(Debug, PartialEq)]
struct FlushCount(usize);

enum Entry<TAnimation> {
	None,
	Some(TAnimation),
	Obsolete((TAnimation, FlushCount)),
}

impl<TAnimation: Debug> Debug for Entry<TAnimation> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::None => write!(f, "None"),
			Self::Some(arg0) => f.debug_tuple("Some").field(arg0).finish(),
			Self::Obsolete(arg0) => f.debug_tuple("Obsolete").field(arg0).finish(),
		}
	}
}

impl<TAnimation: PartialEq> PartialEq for Entry<TAnimation> {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Some(l0), Self::Some(r0)) => l0 == r0,
			(Self::Obsolete(l0), Self::Obsolete(r0)) => l0 == r0,
			_ => core::mem::discriminant(self) == core::mem::discriminant(other),
		}
	}
}

impl<TAnimation> Entry<TAnimation> {
	fn take(&mut self) -> Entry<TAnimation> {
		mem::replace(self, Entry::None)
	}
}

#[derive(Component)]
pub struct AnimationDispatch<TAnimation = Animation>(
	Entry<TAnimation>,
	Entry<TAnimation>,
	Entry<TAnimation>,
);

impl<TAnimation: Debug> Debug for AnimationDispatch<TAnimation> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("AnimationDispatch")
			.field(&self.0)
			.field(&self.1)
			.field(&self.2)
			.finish()
	}
}

impl<TAnimation: PartialEq> PartialEq for AnimationDispatch<TAnimation> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0 && self.1 == other.1 && self.2 == other.2
	}
}

impl<TAnimation> AnimationDispatch<TAnimation> {
	fn slot(&mut self, priority: Priority) -> &mut Entry<TAnimation> {
		match priority {
			Priority::High => &mut self.0,
			Priority::Middle => &mut self.1,
			Priority::Low => &mut self.2,
		}
	}
}

impl<TAnimation> Default for AnimationDispatch<TAnimation> {
	fn default() -> Self {
		Self(Entry::None, Entry::None, Entry::None)
	}
}

impl<TAnimation> HighestPriorityAnimation<TAnimation> for AnimationDispatch<TAnimation> {
	fn highest_priority_animation(&self) -> Option<&TAnimation> {
		match self {
			AnimationDispatch(Entry::Some(animation), ..) => Some(animation),
			AnimationDispatch(_, Entry::Some(animation), _) => Some(animation),
			AnimationDispatch(.., Entry::Some(animation)) => Some(animation),
			_ => None,
		}
	}
}

impl<TAnimation: AnimationChainUpdate + Debug> InsertAnimation<TAnimation>
	for AnimationDispatch<TAnimation>
{
	fn insert(&mut self, mut animation: TAnimation, priority: Priority) {
		let slot = self.slot(priority);

		if let Entry::Some(last) | Entry::Obsolete((last, ..)) = slot {
			animation.chain_update(last);
		}

		*slot = Entry::Some(animation);
	}
}

impl<TAnimation> MarkObsolete for AnimationDispatch<TAnimation> {
	fn mark_obsolete(&mut self, priority: Priority) {
		let slot = self.slot(priority);

		*slot = match slot.take() {
			Entry::Some(animation) => Entry::Obsolete((animation, FlushCount(0))),
			_ => Entry::None,
		}
	}
}

impl<TAnimation> FlushObsolete for AnimationDispatch<TAnimation> {
	fn flush_obsolete(&mut self, priority: Priority) {
		let slot = self.slot(priority);

		*slot = match slot.take() {
			Entry::Obsolete((a, FlushCount(0))) => Entry::Obsolete((a, FlushCount(1))),
			Entry::Obsolete((.., FlushCount(1))) => Entry::None,
			e => e,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::default;

	#[derive(Default, Debug, PartialEq, Clone)]
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
	fn mark_obsolete_low() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("low"), Priority::Low);
		dispatch.mark_obsolete(Priority::Low);

		assert_eq!(None, dispatch.highest_priority_animation());
	}

	#[test]
	fn mark_obsolete_middle() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("middle"), Priority::Middle);
		dispatch.mark_obsolete(Priority::Middle);

		assert_eq!(None, dispatch.highest_priority_animation());
	}

	#[test]
	fn mark_obsolete_high() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("high"), Priority::High);
		dispatch.mark_obsolete(Priority::High);

		assert_eq!(None, dispatch.highest_priority_animation());
	}

	#[test]
	fn call_chain_update() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("last"), Priority::High);
		dispatch.insert(_Animation::new("mock"), Priority::High);

		let mock = dispatch.highest_priority_animation().unwrap();

		assert_eq!(vec![_Animation::new("last")], mock.chain_update_calls);
	}

	#[test]
	fn call_chain_update_on_marked_obsolete() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("last"), Priority::High);
		dispatch.mark_obsolete(Priority::High);
		dispatch.insert(_Animation::new("mock"), Priority::High);

		let mock = dispatch.highest_priority_animation().unwrap();

		assert_eq!(vec![_Animation::new("last")], mock.chain_update_calls);
	}

	#[test]
	fn do_not_call_chain_update_on_marked_obsolete_2_times_ago() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("last"), Priority::High);
		dispatch.mark_obsolete(Priority::High);
		dispatch.mark_obsolete(Priority::High);
		dispatch.insert(_Animation::new("mock"), Priority::High);

		let mock = dispatch.highest_priority_animation().unwrap();

		assert_eq!(vec![] as Vec<_Animation>, mock.chain_update_calls);
	}

	#[test]
	fn do_not_call_chain_update_on_marked_obsolete_after_flushed_twice() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("last"), Priority::High);
		dispatch.mark_obsolete(Priority::High);
		dispatch.flush_obsolete(Priority::High);
		dispatch.flush_obsolete(Priority::High);
		dispatch.insert(_Animation::new("mock"), Priority::High);

		let mock = dispatch.highest_priority_animation().unwrap();

		assert_eq!(vec![] as Vec<_Animation>, mock.chain_update_calls);
	}

	#[test]
	fn call_chain_update_on_marked_obsolete_after_flushed_once() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.insert(_Animation::new("last"), Priority::High);
		dispatch.mark_obsolete(Priority::High);
		dispatch.flush_obsolete(Priority::High);
		dispatch.insert(_Animation::new("mock"), Priority::High);

		let mock = dispatch.highest_priority_animation().unwrap();

		assert_eq!(vec![_Animation::new("last")], mock.chain_update_calls);
	}
}
