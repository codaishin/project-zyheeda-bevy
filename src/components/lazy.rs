use super::{Agent, SpawnBehaviorFn, Spawner};
use bevy::{ecs::system::Commands, math::Ray};

pub type DespawnBehaviorFn = fn(&mut Commands, Agent);

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Lazy {
	run_fn: Option<SpawnBehaviorFn>,
	stop_fn: Option<DespawnBehaviorFn>,
}

impl Lazy {
	pub fn new(run_fn: Option<SpawnBehaviorFn>, stop_fn: Option<DespawnBehaviorFn>) -> Self {
		Self { run_fn, stop_fn }
	}

	pub fn run(&self, commands: &mut Commands, agent: Agent, spawner: Spawner, ray: Ray) {
		let Some(func) = self.run_fn else {
			return;
		};
		func(commands, agent, spawner, ray)
	}

	pub fn stop(&self, commands: &mut Commands, agent: Agent) {
		let Some(func) = self.stop_fn else {
			return;
		};
		func(commands, agent);
	}
}
