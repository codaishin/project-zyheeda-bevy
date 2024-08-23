use bevy::prelude::Entity;
use bevy_rapier3d::prelude::CollisionEvent;
use common::components::ColliderRoot;

use crate::components::blocker::Blocker;

pub(crate) mod damage_health;
pub(crate) mod rapier_context;

pub trait ActOn<TTarget> {
	fn act_on(&mut self, target: &mut TTarget);
}

pub trait FromCollisionEvent {
	fn from_collision<F>(event: &CollisionEvent, get_root: F) -> Self
	where
		F: Fn(Entity) -> ColliderRoot;
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
	fn blockable<const N: usize>(blockers: [Blocker; N]) -> Self;
}
