pub mod simple_movement;

use crate::components::lazy::Lazy;

pub trait ToLazy {
	fn to_lazy() -> Option<Lazy>;
}

#[cfg(test)]
pub mod test_tools {
	use crate::components::{lazy::Lazy, Agent, Spawner};
	use bevy::{ecs::system::Commands, math::Ray};

	pub fn run_lazy(
		behavior: Option<Lazy>,
		agent: Agent,
		spawner: Spawner,
		ray: Ray,
	) -> impl FnMut(Commands) {
		move |mut commands| {
			let Some(behavior) = behavior else {
				return;
			};
			behavior.run(&mut commands, agent, spawner, ray);
		}
	}

	pub fn stop_lazy(behavior: Option<Lazy>, agent: Agent) -> impl FnMut(Commands) {
		move |mut commands| {
			let Some(behavior) = behavior else {
				return;
			};
			behavior.stop(&mut commands, agent);
		}
	}
}
