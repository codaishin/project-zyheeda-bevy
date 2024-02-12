use crate::skill::SelectInfo;
use bevy::{
	ecs::{component::Component, entity::Entity, system::EntityCommands},
	transform::components::{GlobalTransform, Transform},
};

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Spawner(pub GlobalTransform);

#[derive(Debug, PartialEq, Clone)]
pub struct Outdated<TComponent: Component> {
	pub entity: Entity,
	pub component: TComponent,
}

pub type Target = SelectInfo<Outdated<GlobalTransform>>;
pub type TransformFN = fn(&mut Transform, &Spawner, &Target);
pub type StartBehaviorFn = fn(&mut EntityCommands, &Transform, &Spawner, &Target);
pub type StopBehaviorFn = fn(&mut EntityCommands);

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub struct BehaviorMeta {
	pub transform_fn: Option<TransformFN>,
	pub run_fn: Option<StartBehaviorFn>,
	pub stop_fn: Option<StopBehaviorFn>,
}
