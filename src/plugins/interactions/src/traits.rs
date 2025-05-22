pub(crate) mod act_on;
pub(crate) mod rapier_context;

use bevy::prelude::Entity;
use bevy_rapier3d::prelude::CollisionEvent;
use common::blocker::Blocker;

pub trait FromCollisionEvent {
	fn from_collision<F>(event: &CollisionEvent, get_root: F) -> Self
	where
		F: Fn(Entity) -> Entity;
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TrackState {
	Changed,
	Unchanged,
}

pub(crate) trait Track<TEvent> {
	fn track(&mut self, event: &TEvent) -> TrackState;
}

pub(crate) trait Flush {
	type TResult;
	fn flush(&mut self) -> Self::TResult;
}

pub(crate) trait Blockable {
	fn blockable(blockers: &[Blocker]) -> Self;
}
