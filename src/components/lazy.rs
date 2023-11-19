use super::{Agent, SpawnBehaviorFn};
use bevy::ecs::system::Commands;

pub type DespawnBehaviorFn = fn(&mut Commands, Agent);

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Lazy {
	pub run_fn: Option<SpawnBehaviorFn>,
	pub stop_fn: Option<DespawnBehaviorFn>,
}
