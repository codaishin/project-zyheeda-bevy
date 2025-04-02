use crate::traits::{
	AnimationChainUpdate,
	AnimationPlayers,
	AnimationPlayersWithoutTransitions,
	HighestPriorityAnimation,
};
use bevy::prelude::*;
use common::traits::{
	animation::{Animation, AnimationPriority, StartAnimation, StopAnimation},
	track::{IsTracking, Track, Untrack},
};
use std::{
	collections::{HashSet, hash_set::Iter},
	fmt::Debug,
	iter::Cloned,
	mem,
};

#[derive(Default, Debug, PartialEq)]
enum Entry<TAnimation> {
	#[default]
	None,
	Some(TAnimation),
	Obsolete(TAnimation),
}

impl<TAnimation> Entry<TAnimation> {
	fn take(&mut self) -> Entry<TAnimation> {
		mem::replace(self, Entry::None)
	}
}

#[derive(Component, Debug, PartialEq)]
pub struct AnimationDispatch<TAnimation = Animation> {
	pub(crate) animation_players: HashSet<Entity>,
	animation_transitions: HashSet<Entity>,
	stack: (Entry<TAnimation>, Entry<TAnimation>, Entry<TAnimation>),
}

#[cfg(test)]
impl AnimationDispatch {
	pub(crate) fn to<const N: usize>(entities: [Entity; N]) -> Self {
		let mut dispatch = Self::default();
		for entity in entities {
			dispatch.animation_players.insert(entity);
		}

		dispatch
	}
}

impl<TAnimation> AnimationDispatch<TAnimation> {
	fn slot<TLayer>(&mut self, layer: TLayer) -> &mut Entry<TAnimation>
	where
		TLayer: Into<AnimationPriority>,
	{
		match layer.into() {
			AnimationPriority::High => &mut self.stack.0,
			AnimationPriority::Medium => &mut self.stack.1,
			AnimationPriority::Low => &mut self.stack.2,
		}
	}

	fn start_animation<TLayer>(&mut self, layer: TLayer, mut animation: TAnimation)
	where
		TLayer: Into<AnimationPriority>,
		TAnimation: AnimationChainUpdate,
	{
		let slot = self.slot(layer);

		if let Entry::Some(last) | Entry::Obsolete(last) = slot {
			animation.chain_update(last);
		}

		*slot = Entry::Some(animation);
	}
}

impl<TAnimation> Default for AnimationDispatch<TAnimation> {
	fn default() -> Self {
		Self {
			animation_players: default(),
			animation_transitions: default(),
			stack: default(),
		}
	}
}

impl Track<AnimationPlayer> for AnimationDispatch {
	fn track(&mut self, entity: Entity, _: &AnimationPlayer) {
		self.animation_players.insert(entity);
	}
}

impl IsTracking<AnimationPlayer> for AnimationDispatch {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.animation_players.contains(entity)
	}
}

impl Untrack<AnimationPlayer> for AnimationDispatch {
	fn untrack(&mut self, entity: &Entity) {
		self.animation_players.remove(entity);
	}
}

impl Track<AnimationTransitions> for AnimationDispatch {
	fn track(&mut self, entity: Entity, _: &AnimationTransitions) {
		self.animation_transitions.insert(entity);
	}
}

impl IsTracking<AnimationTransitions> for AnimationDispatch {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.animation_transitions.contains(entity)
	}
}

impl Untrack<AnimationTransitions> for AnimationDispatch {
	fn untrack(&mut self, entity: &Entity) {
		self.animation_transitions.remove(entity);
	}
}

impl<'a> AnimationPlayers<'a> for AnimationDispatch {
	type TIter = Cloned<Iter<'a, Entity>>;

	fn animation_players(&'a self) -> Self::TIter {
		self.animation_players.iter().cloned()
	}
}

impl<'a> AnimationPlayersWithoutTransitions<'a> for AnimationDispatch {
	type TIter = IterWithoutTransitions<'a>;

	fn animation_players_without_transition(&'a self) -> Self::TIter {
		IterWithoutTransitions {
			dispatch: self,
			iter: self.animation_players.iter(),
		}
	}
}

pub struct IterWithoutTransitions<'a> {
	dispatch: &'a AnimationDispatch,
	iter: Iter<'a, Entity>,
}

impl Iterator for IterWithoutTransitions<'_> {
	type Item = Entity;

	fn next(&mut self) -> Option<Self::Item> {
		self.iter
			.find(|e| !self.dispatch.animation_transitions.contains(e))
			.cloned()
	}
}

impl<TAnimation> HighestPriorityAnimation<TAnimation> for AnimationDispatch<TAnimation>
where
	TAnimation: Clone,
{
	fn highest_priority_animation(&self) -> Option<TAnimation> {
		match &self.stack {
			(Entry::Some(animation), ..) => Some(animation.clone()),
			(_, Entry::Some(animation), _) => Some(animation.clone()),
			(.., Entry::Some(animation)) => Some(animation.clone()),
			_ => None,
		}
	}
}

impl StartAnimation for AnimationDispatch {
	fn start_animation<TLayer>(&mut self, layer: TLayer, animation: Animation)
	where
		TLayer: Into<AnimationPriority>,
	{
		self.start_animation(layer, animation);
	}
}

impl<TAnimation> StopAnimation for AnimationDispatch<TAnimation> {
	fn stop_animation<TLayer>(&mut self, layer: TLayer)
	where
		TLayer: Into<AnimationPriority>,
	{
		let slot = self.slot(layer);

		*slot = match slot.take() {
			Entry::Some(animation) => Entry::Obsolete(animation),
			_ => Entry::None,
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

	struct _Low;

	impl From<_Low> for AnimationPriority {
		fn from(_: _Low) -> Self {
			AnimationPriority::Low
		}
	}
	struct _Med;

	impl From<_Med> for AnimationPriority {
		fn from(_: _Med) -> Self {
			AnimationPriority::Medium
		}
	}

	struct _High;

	impl From<_High> for AnimationPriority {
		fn from(_: _High) -> Self {
			AnimationPriority::High
		}
	}

	#[test]
	fn insert_low_priority() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_Low, _Animation::new("low"));

		assert_eq!(
			Some(_Animation::new("low")),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn insert_medium_priority() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_Med, _Animation::new("middle"));
		dispatch.start_animation(_Low, _Animation::new("low"));

		assert_eq!(
			Some(_Animation::new("middle")),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn insert_high_priority() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_High, _Animation::new("high"));
		dispatch.start_animation(_Med, _Animation::new("middle"));

		assert_eq!(
			Some(_Animation::new("high")),
			dispatch.highest_priority_animation()
		);
	}

	#[test]
	fn mark_obsolete_low() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_Low, _Animation::new("low"));
		dispatch.stop_animation(_Low);

		assert_eq!(None, dispatch.highest_priority_animation());
	}

	#[test]
	fn mark_obsolete_middle() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_Med, _Animation::new("middle"));
		dispatch.stop_animation(_Med);

		assert_eq!(None, dispatch.highest_priority_animation());
	}

	#[test]
	fn mark_obsolete_high() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_High, _Animation::new("high"));
		dispatch.stop_animation(_High);

		assert_eq!(None, dispatch.highest_priority_animation());
	}

	#[test]
	fn call_chain_update() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_High, _Animation::new("last"));
		dispatch.start_animation(_High, _Animation::new("mock"));

		let mock = dispatch.highest_priority_animation().unwrap();

		assert_eq!(vec![_Animation::new("last")], mock.chain_update_calls);
	}

	#[test]
	fn call_chain_update_on_marked_obsolete() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_High, _Animation::new("last"));
		dispatch.stop_animation(_High);
		dispatch.start_animation(_High, _Animation::new("mock"));

		let mock = dispatch.highest_priority_animation().unwrap();

		assert_eq!(vec![_Animation::new("last")], mock.chain_update_calls);
	}

	#[test]
	fn do_not_call_chain_update_on_marked_obsolete_2_times_ago() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_High, _Animation::new("last"));
		dispatch.stop_animation(_High);
		dispatch.stop_animation(_High);
		dispatch.start_animation(_High, _Animation::new("mock"));

		let mock = dispatch.highest_priority_animation().unwrap();

		assert_eq!(vec![] as Vec<_Animation>, mock.chain_update_calls);
	}

	fn as_track<TComponent>(
		tracker: &mut (impl Track<TComponent> + IsTracking<TComponent> + Untrack<TComponent>),
	) -> &mut (impl Track<TComponent> + IsTracking<TComponent> + Untrack<TComponent>)
	where
		AnimationDispatch: Track<TComponent> + IsTracking<TComponent> + Untrack<TComponent>,
	{
		tracker
	}

	#[test]
	fn track_animation_player() {
		let dispatch = &mut AnimationDispatch::default();
		as_track::<AnimationPlayer>(dispatch)
			.track(Entity::from_raw(1), &AnimationPlayer::default());
		as_track::<AnimationPlayer>(dispatch)
			.track(Entity::from_raw(2), &AnimationPlayer::default());

		assert_eq!(
			HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			dispatch.animation_players
		)
	}

	#[test]
	fn untrack_animation_player() {
		let dispatch = &mut AnimationDispatch {
			animation_players: HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			..default()
		};
		as_track::<AnimationPlayer>(dispatch).untrack(&Entity::from_raw(1));

		assert_eq!(
			HashSet::from([Entity::from_raw(2)]),
			dispatch.animation_players
		)
	}

	#[test]
	fn is_tracking_animation_player() {
		let dispatch = &mut AnimationDispatch {
			animation_players: HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			..default()
		};

		assert_eq!(
			[true, false],
			[
				as_track::<AnimationPlayer>(dispatch).is_tracking(&Entity::from_raw(2)),
				as_track::<AnimationPlayer>(dispatch).is_tracking(&Entity::from_raw(3)),
			]
		)
	}

	#[test]
	fn track_animation_transition() {
		let dispatch = &mut AnimationDispatch::default();
		as_track::<AnimationTransitions>(dispatch)
			.track(Entity::from_raw(1), &AnimationTransitions::default());
		as_track::<AnimationTransitions>(dispatch)
			.track(Entity::from_raw(2), &AnimationTransitions::default());

		assert_eq!(
			HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			dispatch.animation_transitions
		)
	}

	#[test]
	fn untrack_animation_transition() {
		let dispatch = &mut AnimationDispatch {
			animation_transitions: HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			..default()
		};
		as_track::<AnimationTransitions>(dispatch).untrack(&Entity::from_raw(1));

		assert_eq!(
			HashSet::from([Entity::from_raw(2)]),
			dispatch.animation_transitions
		)
	}

	#[test]
	fn is_tracking_animation_transition() {
		let dispatch = &mut AnimationDispatch {
			animation_transitions: HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			..default()
		};

		assert_eq!(
			[true, false],
			[
				as_track::<AnimationTransitions>(dispatch).is_tracking(&Entity::from_raw(2)),
				as_track::<AnimationTransitions>(dispatch).is_tracking(&Entity::from_raw(3)),
			]
		)
	}

	#[test]
	fn iterate_animation_players() {
		let dispatch = AnimationDispatch {
			animation_players: HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			..default()
		};

		assert_eq!(
			HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			dispatch.animation_players().collect::<HashSet<_>>(),
		)
	}

	#[test]
	fn iterate_animation_players_without_transitions() {
		let dispatch = AnimationDispatch {
			animation_players: HashSet::from([
				Entity::from_raw(1),
				Entity::from_raw(2),
				Entity::from_raw(3),
			]),
			animation_transitions: HashSet::from([Entity::from_raw(2)]),
			..default()
		};

		assert_eq!(
			HashSet::from([Entity::from_raw(1), Entity::from_raw(3)]),
			dispatch
				.animation_players_without_transition()
				.collect::<HashSet<_>>(),
		)
	}
}
