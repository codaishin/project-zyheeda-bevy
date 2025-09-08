use crate::{
	events::{Collision, InteractionEvent},
	traits::{Flush, Track, TrackState},
};
use bevy::prelude::{Entity, Resource};
use std::collections::{HashMap, HashSet, hash_map::Entry};

#[derive(Resource, Default)]
pub(crate) struct TrackRayInteractions(HashMap<(Entity, Entity), Refreshed>);

impl Track<InteractionEvent> for TrackRayInteractions {
	fn track(&mut self, InteractionEvent(a, collision): &InteractionEvent) -> TrackState {
		let Collision::Started(b) = collision else {
			return TrackState::Unchanged;
		};

		match self.0.entry((*a, *b)) {
			Entry::Occupied(mut entry) => {
				*entry.get_mut() = Refreshed(true);

				TrackState::Unchanged
			}
			Entry::Vacant(entry) => {
				entry.insert(Refreshed(true));

				TrackState::Changed
			}
		}
	}
}

impl Flush for TrackRayInteractions {
	type TResult = Vec<InteractionEvent>;

	fn flush(&mut self) -> Self::TResult {
		let mut end = HashSet::new();
		let mut retain = HashSet::new();

		for ((a, b), state) in self.0.iter_mut() {
			if state == &Refreshed(false) {
				end.insert((*a, *b));
			} else {
				retain.insert((*a, *b));
			}
			*state = Refreshed(false);
		}

		self.0.retain(|key, _| retain.contains(key));

		end.into_iter()
			.map(|(a, b)| InteractionEvent::of(a).collision(Collision::Ended(b)))
			.collect()
	}
}

#[derive(PartialEq)]
struct Refreshed(bool);

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::Collision;

	#[test]
	fn track_return_change_when_empty() {
		let mut track = TrackRayInteractions::default();
		let a = Entity::from_raw(42);
		let b = Entity::from_raw(43);

		assert_eq!(
			TrackState::Changed,
			track.track(&InteractionEvent::of(a).collision(Collision::Started(b)))
		);
	}

	#[test]
	fn track_return_unchanged_when_tracking_same_interaction_twice() {
		let mut track = TrackRayInteractions::default();
		let a = Entity::from_raw(42);
		let b = Entity::from_raw(43);

		assert_eq!(
			[TrackState::Changed, TrackState::Unchanged],
			[
				track.track(&InteractionEvent::of(a).collision(Collision::Started(b))),
				track.track(&InteractionEvent::of(a).collision(Collision::Started(b)))
			]
		);
	}

	#[test]
	fn flush_returns_empty_when_no_event_tracked() {
		let mut track = TrackRayInteractions::default();

		assert_eq!(vec![] as Vec<InteractionEvent>, track.flush());
	}

	#[test]
	fn flush_returns_end_event_for_start_event_that_have_been_slushed_twice() {
		let mut track = TrackRayInteractions::default();
		let a = Entity::from_raw(42);
		let b = Entity::from_raw(43);

		track.track(&InteractionEvent::of(a).collision(Collision::Started(b)));

		assert_eq!(
			[
				vec![],
				vec![InteractionEvent::of(a).collision(Collision::Ended(b))]
			],
			[track.flush(), track.flush()]
		)
	}

	#[test]
	fn flush_returns_no_end_events_when_start_event_refreshed() {
		let mut track = TrackRayInteractions::default();
		let a = Entity::from_raw(42);
		let b = Entity::from_raw(43);

		assert_eq!(
			[
				vec![] as Vec<InteractionEvent>,
				vec![] as Vec<InteractionEvent>
			],
			[
				{
					track.track(&InteractionEvent::of(a).collision(Collision::Started(b)));
					track.flush()
				},
				{
					track.track(&InteractionEvent::of(a).collision(Collision::Started(b)));
					track.flush()
				}
			]
		)
	}

	#[test]
	fn flush_removes_start_events_for_which_end_events_have_been_produced() {
		let mut track = TrackRayInteractions::default();
		let a = Entity::from_raw(42);
		let b = Entity::from_raw(43);

		track.track(&InteractionEvent::of(a).collision(Collision::Started(b)));

		assert_eq!(
			[
				vec![],
				vec![InteractionEvent::of(a).collision(Collision::Ended(b))],
				vec![]
			],
			[track.flush(), track.flush(), track.flush()]
		)
	}
}
