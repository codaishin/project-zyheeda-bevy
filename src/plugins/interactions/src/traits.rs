use bevy::prelude::{Bundle, Component, Entity};
use bevy_rapier3d::prelude::CollisionEvent;
use common::components::ColliderRoot;

pub(crate) mod damage_health;
pub(crate) mod rapier_context;

pub trait ActOn<TTarget> {
	fn act_on(&mut self, target: &mut TTarget);
}

pub trait ConcatBlockers {
	fn and<TBlocker: Component>(self) -> impl ConcatBlockers + Bundle;
}

pub trait FromCollisionEvent {
	fn from_collision<F>(event: &CollisionEvent, get_root: F) -> Self
	where
		F: Fn(Entity) -> ColliderRoot;
}

#[derive(Debug, PartialEq)]
pub(crate) enum TrackState {
	Changed,
	Unchanged,
}

pub(crate) trait Track<TEvent> {
	fn update(&mut self, event: &TEvent) -> TrackState;
}
