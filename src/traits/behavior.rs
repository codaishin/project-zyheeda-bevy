pub mod projectile;
pub mod simple_movement;
pub mod sword;

use crate::behaviors::meta::BehaviorMeta;

pub trait GetBehaviorMeta {
	fn behavior() -> BehaviorMeta;
}

#[cfg(test)]
pub mod test_tools {
	use super::*;
	use crate::behaviors::meta::Spawner;
	use bevy::{
		ecs::system::Commands,
		math::Ray,
		prelude::Entity,
		transform::components::Transform,
	};

	pub fn run_lazy(
		behavior: BehaviorMeta,
		agent: Entity,
		agent_transform: Transform,
		spawner: Spawner,
		ray: Ray,
	) -> impl FnMut(Commands) {
		move |mut commands| {
			let Some(run) = behavior.run_fn else {
				return;
			};
			let mut agent = commands.entity(agent);
			run(&mut agent, &agent_transform, &spawner, &ray);
		}
	}

	pub fn stop_lazy(behavior: BehaviorMeta, agent: Entity) -> impl FnMut(Commands) {
		move |mut commands| {
			let Some(stop) = behavior.stop_fn else {
				return;
			};
			stop(&mut commands.entity(agent));
		}
	}
}
