use crate::traits::{AnimationPlayers, AnimationPlayersWithoutTransitions, GetActiveAnimations};
use bevy::prelude::*;
use common::traits::{
	animation::{Animation, AnimationPriority, StartAnimation, StopAnimation},
	track::{IsTracking, Track, Untrack},
};
use std::{
	collections::{HashSet, hash_set::Iter},
	fmt::Debug,
	hash::Hash,
	iter::Cloned,
};

#[derive(Component, Debug, PartialEq)]
pub struct AnimationDispatch<TAnimation = Animation>
where
	TAnimation: Eq + Hash,
{
	pub(crate) animation_players: HashSet<Entity>,
	animation_transitions: HashSet<Entity>,
	stack: (
		HashSet<TAnimation>,
		HashSet<TAnimation>,
		HashSet<TAnimation>,
	),
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

impl<TAnimation> AnimationDispatch<TAnimation>
where
	TAnimation: Eq + Hash,
{
	fn slot_mut<TLayer>(&mut self, layer: TLayer) -> &mut HashSet<TAnimation>
	where
		TLayer: Into<AnimationPriority>,
	{
		match layer.into() {
			AnimationPriority::High => &mut self.stack.0,
			AnimationPriority::Medium => &mut self.stack.1,
			AnimationPriority::Low => &mut self.stack.2,
		}
	}

	fn slot<TLayer>(&self, layer: TLayer) -> &HashSet<TAnimation>
	where
		TLayer: Into<AnimationPriority>,
	{
		match layer.into() {
			AnimationPriority::High => &self.stack.0,
			AnimationPriority::Medium => &self.stack.1,
			AnimationPriority::Low => &self.stack.2,
		}
	}

	fn start_animation<TLayer>(&mut self, layer: TLayer, animation: TAnimation)
	where
		TLayer: Into<AnimationPriority>,
	{
		self.slot_mut(layer).insert(animation);
	}
}

impl<TAnimation> Default for AnimationDispatch<TAnimation>
where
	TAnimation: Eq + Hash,
{
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

impl<TAnimation> GetActiveAnimations<TAnimation> for AnimationDispatch<TAnimation>
where
	TAnimation: Clone + Eq + Hash,
{
	type TIter<'a>
		= Iter<'a, TAnimation>
	where
		Self: 'a,
		TAnimation: 'a;

	fn get_active_animations<TPriority>(&self, priority: TPriority) -> Self::TIter<'_>
	where
		TPriority: Into<AnimationPriority>,
	{
		self.slot(priority).iter()
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

impl<TAnimation> StopAnimation for AnimationDispatch<TAnimation>
where
	TAnimation: Eq + Hash,
{
	fn stop_animation<TLayer>(&mut self, layer: TLayer)
	where
		TLayer: Into<AnimationPriority>,
	{
		self.slot_mut(layer).clear();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::default;

	#[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
	struct _Animation {
		name: &'static str,
	}

	impl _Animation {
		fn new(name: &'static str) -> Self {
			Self { name }
		}
	}

	struct _Lo;

	impl From<_Lo> for AnimationPriority {
		fn from(_: _Lo) -> Self {
			AnimationPriority::Low
		}
	}
	struct _Me;

	impl From<_Me> for AnimationPriority {
		fn from(_: _Me) -> Self {
			AnimationPriority::Medium
		}
	}

	struct _Hi;

	impl From<_Hi> for AnimationPriority {
		fn from(_: _Hi) -> Self {
			AnimationPriority::High
		}
	}

	#[test]
	fn insert_priorities() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_Lo, _Animation::new("low/1"));
		dispatch.start_animation(_Lo, _Animation::new("low/2"));
		dispatch.start_animation(_Me, _Animation::new("medium/1"));
		dispatch.start_animation(_Me, _Animation::new("medium/2"));
		dispatch.start_animation(_Hi, _Animation::new("high/1"));
		dispatch.start_animation(_Hi, _Animation::new("high/2"));

		assert_eq!(
			[
				HashSet::from([(&_Animation::new("low/1")), &_Animation::new("low/2")]),
				HashSet::from([(&_Animation::new("medium/1")), &_Animation::new("medium/2")]),
				HashSet::from([(&_Animation::new("high/1")), &_Animation::new("high/2")]),
			],
			[
				dispatch.get_active_animations(_Lo).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Me).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Hi).collect::<HashSet<_>>(),
			]
		);
	}

	#[test]
	fn stop_animations_low() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_Lo, _Animation::new("low"));
		dispatch.start_animation(_Me, _Animation::new("medium"));
		dispatch.start_animation(_Hi, _Animation::new("high"));
		dispatch.stop_animation(_Lo);

		assert_eq!(
			[
				HashSet::from([]),
				HashSet::from([&_Animation::new("medium")]),
				HashSet::from([&_Animation::new("high")]),
			],
			[
				dispatch.get_active_animations(_Lo).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Me).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Hi).collect::<HashSet<_>>(),
			]
		);
	}

	#[test]
	fn stop_animations_medium() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_Lo, _Animation::new("low"));
		dispatch.start_animation(_Me, _Animation::new("medium"));
		dispatch.start_animation(_Hi, _Animation::new("high"));
		dispatch.stop_animation(_Me);

		assert_eq!(
			[
				HashSet::from([&_Animation::new("low")]),
				HashSet::from([]),
				HashSet::from([&_Animation::new("high")]),
			],
			[
				dispatch.get_active_animations(_Lo).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Me).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Hi).collect::<HashSet<_>>(),
			]
		);
	}
	#[test]
	fn stop_animations_high() {
		let mut dispatch = AnimationDispatch::default();
		dispatch.start_animation(_Lo, _Animation::new("low"));
		dispatch.start_animation(_Me, _Animation::new("medium"));
		dispatch.start_animation(_Hi, _Animation::new("high"));
		dispatch.stop_animation(_Hi);

		assert_eq!(
			[
				HashSet::from([&_Animation::new("low")]),
				HashSet::from([&_Animation::new("medium")]),
				HashSet::from([]),
			],
			[
				dispatch.get_active_animations(_Lo).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Me).collect::<HashSet<_>>(),
				dispatch.get_active_animations(_Hi).collect::<HashSet<_>>(),
			]
		);
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
