pub mod projectile;
pub mod simple_movement;

use crate::components::lazy::Lazy;

pub trait ToLazy {
	fn to_lazy() -> Lazy;
}

#[cfg(test)]
pub mod test_tools {
	use crate::components::{lazy::Lazy, Agent, Spawner};
	use bevy::{ecs::system::Commands, math::Ray};

	pub fn run_lazy(
		behavior: Lazy,
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

	pub fn stop_lazy(behavior: Lazy, agent: Agent) -> impl FnMut(Commands) {
		move |mut commands| {
			let Some(stop) = behavior.stop_fn else {
				return;
			};
			stop(&mut commands, agent);
		}
	}
}