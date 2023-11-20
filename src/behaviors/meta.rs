use ::bevy::prelude::Entity;
use bevy::{
	ecs::system::Commands,
	math::Ray,
	transform::components::{GlobalTransform, Transform},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Agent(pub Entity);

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Spawner(pub GlobalTransform);

pub type StopBehaviorFn = fn(&mut Commands, &Agent);
pub type StartBehaviorFn = fn(&mut Commands, &Agent, &Spawner, &Ray);
pub type TransformFN = fn(&mut Transform, &Spawner, &Ray);

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub struct BehaviorMeta {
	pub run_fn: Option<StartBehaviorFn>,
	pub stop_fn: Option<StopBehaviorFn>,
	pub transform_fn: Option<TransformFN>,
}
