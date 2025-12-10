pub(crate) mod act_on;
pub(crate) mod query_filter_definition;
pub(crate) mod rapier_context;
pub(crate) mod ray_cast;
pub(crate) mod update_blockers;

use bevy::prelude::Entity;
use bevy_rapier3d::prelude::CollisionEvent;

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
