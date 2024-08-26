use crate::components::blocker::Blocker;
use bevy::prelude::Entity;
use bevy_rapier3d::prelude::CollisionEvent;
use common::components::ColliderRoot;

pub(crate) mod rapier_context;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ActionType {
	Once,
	OncePerTarget,
	Always,
}

pub trait ActOn<TTarget> {
	fn act_on(&mut self, target: &mut TTarget) -> ActionType;
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
