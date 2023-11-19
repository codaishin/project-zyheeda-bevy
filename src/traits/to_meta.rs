use crate::behaviors::meta::BehaviorMeta;

pub mod projectile;
pub mod simple_movement;

pub trait ToMeta {
	fn meta() -> BehaviorMeta;
}

#[cfg(test)]
pub mod test_tools {
	use super::*;
	use crate::behaviors::meta::{Agent, Spawner};
	use bevy::{ecs::system::Commands, math::Ray};

	pub fn run_lazy(
		behavior: BehaviorMeta,
		agent: Agent,
		spawner: Spawner,
		ray: Ray,
	) -> impl FnMut(Commands) {
		move |mut commands| {
			let Some(run) = behavior.run_fn else {
				return;
			};
			run(&mut commands, agent, spawner, ray);
		}
	}

	pub fn stop_lazy(behavior: BehaviorMeta, agent: Agent) -> impl FnMut(Commands) {
		move |mut commands| {
			let Some(stop) = behavior.stop_fn else {
				return;
			};
			stop(&mut commands, agent);
		}
	}
}
