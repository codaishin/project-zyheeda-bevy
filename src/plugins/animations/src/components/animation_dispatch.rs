mod dto;

use crate::{
	components::animation_dispatch::dto::AnimationDispatchDto,
	traits::{
		AnimationPlayers,
		AnimationPlayersWithoutGraph,
		GetActiveAnimations,
		GetAllActiveAnimations,
	},
};
use bevy::prelude::*;
use common::traits::{
	handles_animations::{AnimationKey, AnimationPriority},
	track::{IsTracking, Track, Untrack},
};
use macros::SavableComponent;
use std::{
	collections::{
		HashSet,
		hash_set::{IntoIter, Iter},
	},
	fmt::Debug,
};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[savable_component(dto = AnimationDispatchDto)]
pub struct AnimationDispatch {
	pub(crate) animation_players: HashSet<Entity>,
	animation_handles: HashSet<Entity>,
	priorities: (
		HashSet<AnimationKey>,
		HashSet<AnimationKey>,
		HashSet<AnimationKey>,
	),
}

impl AnimationDispatch {
	pub(crate) fn slot_mut<TLayer>(&mut self, layer: TLayer) -> &mut HashSet<AnimationKey>
	where
		TLayer: Into<AnimationPriority>,
	{
		match layer.into() {
			AnimationPriority::High => &mut self.priorities.0,
			AnimationPriority::Medium => &mut self.priorities.1,
			AnimationPriority::Low => &mut self.priorities.2,
		}
	}

	pub(crate) fn slot<TLayer>(&self, layer: TLayer) -> &HashSet<AnimationKey>
	where
		TLayer: Into<AnimationPriority>,
	{
		match layer.into() {
			AnimationPriority::High => &self.priorities.0,
			AnimationPriority::Medium => &self.priorities.1,
			AnimationPriority::Low => &self.priorities.2,
		}
	}
}

impl Default for AnimationDispatch {
	fn default() -> Self {
		Self {
			animation_players: default(),
			animation_handles: default(),
			priorities: default(),
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

impl Track<AnimationGraphHandle> for AnimationDispatch {
	fn track(&mut self, entity: Entity, _: &AnimationGraphHandle) {
		self.animation_handles.insert(entity);
	}
}

impl IsTracking<AnimationGraphHandle> for AnimationDispatch {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.animation_handles.contains(entity)
	}
}

impl Untrack<AnimationGraphHandle> for AnimationDispatch {
	fn untrack(&mut self, entity: &Entity) {
		self.animation_handles.remove(entity);
	}
}

impl AnimationPlayers for AnimationDispatch {
	type TIter = IntoIter<Entity>;

	fn animation_players(&self) -> Self::TIter {
		self.animation_players.clone().into_iter()
	}
}

impl AnimationPlayersWithoutGraph for AnimationDispatch {
	type TIter = std::vec::IntoIter<Entity>;

	fn animation_players_without_graph(&self) -> Self::TIter {
		let entities = self
			.animation_players
			.iter()
			.filter(|e| !self.animation_handles.contains(e))
			.copied()
			.collect::<Vec<_>>();
		entities.into_iter()
	}
}

impl GetActiveAnimations for AnimationDispatch {
	type TIter<'a>
		= Iter<'a, AnimationKey>
	where
		Self: 'a;

	fn get_active_animations<TPriority>(&self, priority: TPriority) -> Self::TIter<'_>
	where
		TPriority: Into<AnimationPriority>,
	{
		self.slot(priority).iter()
	}
}

impl GetAllActiveAnimations for AnimationDispatch {
	type TIter<'a>
		= IterAllAnimations<'a>
	where
		Self: 'a;

	fn get_all_active_animations(&self) -> Self::TIter<'_> {
		IterAllAnimations(
			self.priorities.0.iter(),
			self.priorities.1.iter(),
			self.priorities.2.iter(),
		)
	}
}

pub struct IterAllAnimations<'a>(
	Iter<'a, AnimationKey>,
	Iter<'a, AnimationKey>,
	Iter<'a, AnimationKey>,
);

impl<'a> Iterator for IterAllAnimations<'a> {
	type Item = &'a AnimationKey;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(next) = self.0.next() {
			return Some(next);
		}
		if let Some(next) = self.1.next() {
			return Some(next);
		}
		if let Some(next) = self.2.next() {
			return Some(next);
		}

		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::default;
	use common::tools::action_key::slot::SlotKey;

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
	fn track_animation_graph() {
		let dispatch = &mut AnimationDispatch::default();
		as_track::<AnimationGraphHandle>(dispatch)
			.track(Entity::from_raw(1), &AnimationGraphHandle::default());
		as_track::<AnimationGraphHandle>(dispatch)
			.track(Entity::from_raw(2), &AnimationGraphHandle::default());

		assert_eq!(
			HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			dispatch.animation_handles
		)
	}

	#[test]
	fn untrack_animation_graph() {
		let dispatch = &mut AnimationDispatch {
			animation_handles: HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			..default()
		};
		as_track::<AnimationGraphHandle>(dispatch).untrack(&Entity::from_raw(1));

		assert_eq!(
			HashSet::from([Entity::from_raw(2)]),
			dispatch.animation_handles
		)
	}

	#[test]
	fn is_tracking_animation_graph() {
		let dispatch = &mut AnimationDispatch {
			animation_handles: HashSet::from([Entity::from_raw(1), Entity::from_raw(2)]),
			..default()
		};

		assert_eq!(
			[true, false],
			[
				as_track::<AnimationGraphHandle>(dispatch).is_tracking(&Entity::from_raw(2)),
				as_track::<AnimationGraphHandle>(dispatch).is_tracking(&Entity::from_raw(3)),
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
			animation_handles: HashSet::from([Entity::from_raw(2)]),
			..default()
		};

		assert_eq!(
			HashSet::from([Entity::from_raw(1), Entity::from_raw(3)]),
			dispatch
				.animation_players_without_graph()
				.collect::<HashSet<_>>(),
		)
	}

	#[test]
	fn iter_all() {
		let dispatch = AnimationDispatch {
			animation_players: default(),
			animation_handles: default(),
			priorities: (
				HashSet::from([
					AnimationKey::Skill(SlotKey(1)),
					AnimationKey::Skill(SlotKey(2)),
					AnimationKey::Skill(SlotKey(3)),
				]),
				HashSet::from([
					AnimationKey::Skill(SlotKey(4)),
					AnimationKey::Skill(SlotKey(5)),
					AnimationKey::Skill(SlotKey(6)),
				]),
				HashSet::from([
					AnimationKey::Skill(SlotKey(7)),
					AnimationKey::Skill(SlotKey(9)),
					AnimationKey::Skill(SlotKey(9)),
				]),
			),
		};

		assert_eq!(
			HashSet::from([
				AnimationKey::Skill(SlotKey(1)),
				AnimationKey::Skill(SlotKey(2)),
				AnimationKey::Skill(SlotKey(3)),
				AnimationKey::Skill(SlotKey(4)),
				AnimationKey::Skill(SlotKey(5)),
				AnimationKey::Skill(SlotKey(6)),
				AnimationKey::Skill(SlotKey(7)),
				AnimationKey::Skill(SlotKey(9)),
				AnimationKey::Skill(SlotKey(9)),
			]),
			dispatch
				.get_all_active_animations()
				.copied()
				.collect::<HashSet<_>>()
		)
	}
}
