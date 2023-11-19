use ::bevy::prelude::Entity;
use bevy::{ecs::system::Commands, math::Ray, transform::components::GlobalTransform};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Agent(pub Entity);

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Spawner(pub GlobalTransform);

pub type StopBehaviorFn = fn(&mut Commands, Agent);
pub type StartBehaviorFn = fn(&mut Commands, Agent, Spawner, Ray);

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct BehaviorMeta {
	pub run_fn: Option<StartBehaviorFn>,
	pub stop_fn: Option<StopBehaviorFn>,
}
