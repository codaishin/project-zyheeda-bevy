mod interactions_count;
mod sorted_entities;

use crate::{
	events::{Collision, InteractionEvent},
	traits::{Track, TrackState},
};
use bevy::prelude::{Entity, Resource};
use common::components::collider_relations::ChildColliderOf;
use interactions_count::{InteractionsCount, RemainingInteractions};
use sorted_entities::SortedEntities;
use std::collections::{HashMap, hash_map::Entry};

#[derive(Resource, Debug, PartialEq, Default)]
pub(crate) struct TrackInteractionDuplicates(HashMap<SortedEntities, InteractionsCount>);

impl Track<InteractionEvent> for TrackInteractionDuplicates {
	fn track(
		&mut self,
		InteractionEvent(ChildColliderOf(a), collision): &InteractionEvent,
	) -> TrackState {
		match collision {
			Collision::Started(ChildColliderOf(b)) => self.track_started(a, b),
			Collision::Ended(ChildColliderOf(b)) => self.track_ended(a, b),
		}
	}
}

impl TrackInteractionDuplicates {
	fn track_started(&mut self, a: &Entity, b: &Entity) -> TrackState {
		match self.0.entry(SortedEntities::from([*a, *b])) {
			Entry::Vacant(entry) => {
				entry.insert(InteractionsCount::one());
				TrackState::Changed
			}
			Entry::Occupied(mut entry) => {
				let interactions_count = entry.get_mut();
				interactions_count.increment();
				TrackState::Unchanged
			}
		}
	}

	fn track_ended(&mut self, a: &Entity, b: &Entity) -> TrackState {
		let Entry::Occupied(mut entry) = self.0.entry(SortedEntities::from([*a, *b])) else {
			return TrackState::Unchanged;
		};

		let interactions_count = entry.get_mut();
		match interactions_count.try_decrement() {
			RemainingInteractions::None => {
				entry.remove();
				TrackState::Changed
			}
			RemainingInteractions::Some(remaining) => {
				*interactions_count = remaining;
				TrackState::Unchanged
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{events::Collision, traits::TrackState};

	#[test]
	fn single_start() {
		let mut track = TrackInteractionDuplicates::default();
		let start = InteractionEvent::of(ChildColliderOf(Entity::from_raw(42)))
			.collision(Collision::Started(ChildColliderOf(Entity::from_raw(43))));

		assert_eq!(TrackState::Changed, track.track(&start));
	}

	#[test]
	fn multiple_starts() {
		let mut tracker = TrackInteractionDuplicates::default();
		let start = InteractionEvent::of(ChildColliderOf(Entity::from_raw(42)))
			.collision(Collision::Started(ChildColliderOf(Entity::from_raw(43))));

		assert_eq!(
			[TrackState::Changed, TrackState::Unchanged],
			[tracker.track(&start), tracker.track(&start)]
		);
	}

	#[test]
	fn multiple_starts_of_different_interactions() {
		let mut tracker = TrackInteractionDuplicates::default();
		let start_a = InteractionEvent::of(ChildColliderOf(Entity::from_raw(42)))
			.collision(Collision::Started(ChildColliderOf(Entity::from_raw(43))));
		let start_b = InteractionEvent::of(ChildColliderOf(Entity::from_raw(1)))
			.collision(Collision::Started(ChildColliderOf(Entity::from_raw(2))));

		assert_eq!(
			[TrackState::Changed, TrackState::Changed],
			[tracker.track(&start_a), tracker.track(&start_b)]
		);
	}

	#[test]
	fn single_end() {
		let mut tracker = TrackInteractionDuplicates::default();
		let end = InteractionEvent::of(ChildColliderOf(Entity::from_raw(42)))
			.collision(Collision::Ended(ChildColliderOf(Entity::from_raw(43))));

		assert_eq!([TrackState::Unchanged], [tracker.track(&end)]);
	}

	#[test]
	fn start_end() {
		let mut tracker = TrackInteractionDuplicates::default();
		let start = InteractionEvent::of(ChildColliderOf(Entity::from_raw(42)))
			.collision(Collision::Started(ChildColliderOf(Entity::from_raw(43))));
		let end = InteractionEvent::of(ChildColliderOf(Entity::from_raw(42)))
			.collision(Collision::Ended(ChildColliderOf(Entity::from_raw(43))));

		assert_eq!(
			[TrackState::Changed, TrackState::Changed],
			[tracker.track(&start), tracker.track(&end)]
		);
	}

	#[test]
	fn start_start_end() {
		let mut tracker = TrackInteractionDuplicates::default();
		let start = InteractionEvent::of(ChildColliderOf(Entity::from_raw(42)))
			.collision(Collision::Started(ChildColliderOf(Entity::from_raw(43))));
		let end = InteractionEvent::of(ChildColliderOf(Entity::from_raw(42)))
			.collision(Collision::Ended(ChildColliderOf(Entity::from_raw(43))));

		assert_eq!(
			[
				TrackState::Changed,
				TrackState::Unchanged,
				TrackState::Unchanged,
			],
			[
				tracker.track(&start),
				tracker.track(&start),
				tracker.track(&end),
			]
		);
	}
}
