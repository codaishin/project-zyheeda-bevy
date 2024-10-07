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
use bevy::prelude::*;
use common::traits::track::{IsTracking, Track, Untrack};
use std::{collections::HashSet, fmt::Debug, mem};

#[derive(Debug, PartialEq)]
struct FlushCount(usize);

#[derive(Default, Debug, PartialEq)]
enum Entry<TAnimation> {
	#[default]
	None,
	Some(TAnimation),
	Obsolete((TAnimation, FlushCount)),
}

impl<TAnimation> Entry<TAnimation> {
	fn take(&mut self) -> Entry<TAnimation> {
		mem::replace(self, Entry::None)
	}
}

#[derive(Component, Debug, PartialEq)]
pub struct AnimationDispatch<TAnimation = Animation> {
	pub(crate) animation_players: HashSet<Entity>,
	stack: (Entry<TAnimation>, Entry<TAnimation>, Entry<TAnimation>),
}

impl<TAnimation> AnimationDispatch<TAnimation> {
	fn slot(&mut self, priority: Priority) -> &mut Entry<TAnimation> {
		match priority {
			Priority::High => &mut self.stack.0,
			Priority::Middle => &mut self.stack.1,
			Priority::Low => &mut self.stack.2,
		}
	}
}

impl<TAnimation> Default for AnimationDispatch<TAnimation> {
	fn default() -> Self {
		Self {
			animation_players: default(),
			stack: default(),
		}
	}
}

impl<TAnimation> Track<AnimationPlayer> for AnimationDispatch<TAnimation> {
	fn track(&mut self, entity: Entity) {
		self.animation_players.insert(entity);
	}
}

impl<TAnimation> IsTracking<AnimationPlayer> for AnimationDispatch<TAnimation> {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.animation_players.contains(entity)
	}
}

impl<TAnimation> Untrack<AnimationPlayer> for AnimationDispatch<TAnimation> {
	fn untrack(&mut self, entity: &Entity) {
		self.animation_players.remove(entity);
	}
}

impl<TAnimation> HighestPriorityAnimation<TAnimation> for AnimationDispatch<TAnimation> {
	fn highest_priority_animation(&self) -> Option<&TAnimation> {
		match &self.stack {
			(Entry::Some(animation), ..) => Some(animation),
			(_, Entry::Some(animation), _) => Some(animation),
			(.., Entry::Some(animation)) => Some(animation),
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

	#[test]
	fn track_animation_player() {
		let mut dispatch = AnimationDispatch::<_Animation>::default();
		dispatch.track(Entity::from_raw(1));
		dispatch.track(Entity::from_raw(2));

		assert_eq!(
			HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			dispatch.animation_players
		)
	}

	#[test]
	fn untrack_animation_player() {
		let mut dispatch = AnimationDispatch::<_Animation> {
			animation_players: HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			..default()
		};
		dispatch.untrack(&Entity::from_raw(1));

		assert_eq!(
			HashSet::from([Entity::from_raw(2)]),
			dispatch.animation_players
		)
	}

	#[test]
	fn is_tracking_animation_player() {
		let dispatch = AnimationDispatch::<_Animation> {
			animation_players: HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			..default()
		};

		assert_eq!(
			[true, false],
			[
				dispatch.is_tracking(&Entity::from_raw(2)),
				dispatch.is_tracking(&Entity::from_raw(3))
			]
		)
	}
}
