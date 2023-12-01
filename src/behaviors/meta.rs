use bevy::{
	ecs::system::EntityCommands,
	math::Ray,
	transform::components::{GlobalTransform, Transform},
};

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Spawner(pub GlobalTransform);

pub type TransformFN = fn(&mut Transform, &Spawner, &Ray);
pub type StartBehaviorFn = fn(&mut EntityCommands, &Spawner, &Ray);
pub type StopBehaviorFn = fn(&mut EntityCommands);

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub struct BehaviorMeta {
	pub transform_fn: Option<TransformFN>,
	pub run_fn: Option<StartBehaviorFn>,
	pub stop_fn: Option<StopBehaviorFn>,
}
