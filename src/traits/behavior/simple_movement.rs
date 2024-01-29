use super::GetBehaviorMeta;
use crate::{
	behaviors::meta::{BehaviorMeta, Spawner, Target},
	components::SimpleMovement,
};
use bevy::{ecs::system::EntityCommands, math::Vec3, transform::components::Transform};

fn run_fn(
	agent: &mut EntityCommands,
	_agent_transform: &Transform,
	_spawner: &Spawner,
	target: &Target,
) {
	let ray = target.ray;
	let Some(length) = ray.intersect_plane(Vec3::ZERO, Vec3::Y) else {
		return;
	};
	let target = ray.origin + ray.direction * length;
	agent.insert(SimpleMovement { target });
}

fn stop_fn(agent: &mut EntityCommands) {
	agent.remove::<SimpleMovement>();
}

impl GetBehaviorMeta for SimpleMovement {
	fn behavior() -> BehaviorMeta {
		BehaviorMeta {
			run_fn: Some(run_fn),
			stop_fn: Some(stop_fn),
			transform_fn: None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		test_tools::utils::assert_eq_approx,
		traits::behavior::test_tools::{run_lazy, stop_lazy},
	};
	use bevy::{
		prelude::{App, Ray, Update, Vec3},
		utils::default,
	};

	#[test]
	fn move_to_zero() {
		let mut app = App::new();
		let behavior = SimpleMovement::behavior();
		let agent = app.world.spawn(()).id();
		let target = Target {
			ray: Ray {
				origin: Vec3::Y,
				direction: Vec3::NEG_Y,
			},
			..default()
		};

		app.add_systems(
			Update,
			run_lazy(behavior, agent, default(), Spawner::default(), target),
		);
		app.update();

		let movement = app.world.entity(agent).get::<SimpleMovement>();

		assert_eq!(Some(Vec3::ZERO), movement.map(|m| m.target));
	}

	#[test]
	fn move_to_offset() {
		let mut app = App::new();
		let behavior = SimpleMovement::behavior();
		let agent = app.world.spawn(()).id();
		let target = Target {
			ray: Ray {
				origin: Vec3::ONE,
				direction: Vec3::NEG_Y,
			},
			..default()
		};

		app.add_systems(
			Update,
			run_lazy(behavior, agent, default(), Spawner::default(), target),
		);
		app.update();

		let movement = app.world.entity(agent).get::<SimpleMovement>().unwrap();

		assert_eq!(Vec3::new(1., 0., 1.), movement.target);
	}

	#[test]
	fn move_to_offset_2() {
		let mut app = App::new();
		let behavior = SimpleMovement::behavior();
		let agent = app.world.spawn(()).id();
		let target = Target {
			ray: Ray {
				origin: Vec3::new(0., 4., 0.),
				direction: Vec3::new(0., -4., 3.),
			},
			..default()
		};

		app.add_systems(
			Update,
			run_lazy(behavior, agent, default(), Spawner::default(), target),
		);
		app.update();

		let movement = app.world.entity(agent).get::<SimpleMovement>().unwrap();

		assert_eq_approx!(Vec3::new(0., 0., 3.), movement.target, 0.000001);
	}

	#[test]
	fn remove_simple_movement() {
		let mut app = App::new();
		let behavior = SimpleMovement::behavior();
		let agent = app
			.world
			.spawn((SimpleMovement { target: Vec3::ZERO },))
			.id();

		app.add_systems(Update, stop_lazy(behavior, agent));
		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<SimpleMovement>());
	}
}
