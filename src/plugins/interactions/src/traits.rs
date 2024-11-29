use bevy::prelude::Entity;
use bevy_rapier3d::prelude::CollisionEvent;
use common::{blocker::Blocker, components::ColliderRoot, effects::EffectApplies};
use std::time::Duration;

pub(crate) mod rapier_context;

pub trait ActOn<TTarget> {
	fn act(&mut self, self_entity: Entity, target: &mut TTarget, delta: Duration) -> EffectApplies;
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
