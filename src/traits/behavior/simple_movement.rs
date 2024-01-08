use super::GetBehaviorMeta;
use crate::{
	behaviors::meta::{BehaviorMeta, Spawner},
	components::{Marker, SimpleMovement},
	markers::{Fast, Slow},
};
use bevy::{
	ecs::system::EntityCommands,
	math::{Ray, Vec3},
	transform::components::Transform,
};

fn run_fn(agent: &mut EntityCommands, _agent_transform: &Transform, _spawner: &Spawner, ray: &Ray) {
	let Some(length) = ray.intersect_plane(Vec3::ZERO, Vec3::Y) else {
		return;
	};
	agent.insert(SimpleMovement {
		target: ray.origin + ray.direction * length,
	});
}

fn stop_fn(agent: &mut EntityCommands) {
	agent.remove::<(SimpleMovement, Marker<Fast>, Marker<Slow>)>();
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
		test_tools::assert_eq_approx,
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
		let ray = Ray {
			origin: Vec3::Y,
			direction: Vec3::NEG_Y,
		};

		app.add_systems(
			Update,
			run_lazy(behavior, agent, default(), Spawner::default(), ray),
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
		let ray = Ray {
			origin: Vec3::ONE,
			direction: Vec3::NEG_Y,
		};

		app.add_systems(
			Update,
			run_lazy(behavior, agent, default(), Spawner::default(), ray),
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
		let ray = Ray {
			origin: Vec3::new(0., 4., 0.),
			direction: Vec3::new(0., -4., 3.),
		};

		app.add_systems(
			Update,
			run_lazy(behavior, agent, default(), Spawner::default(), ray),
		);
		app.update();

		let movement = app.world.entity(agent).get::<SimpleMovement>().unwrap();

		assert_eq_approx!(Vec3::new(0., 0., 3.), movement.target, 0.000001);
	}

	#[test]
	fn remove_all_relevant_components() {
		let mut app = App::new();
		let behavior = SimpleMovement::behavior();
		let agent = app
			.world
			.spawn((
				SimpleMovement { target: Vec3::ZERO },
				Marker::<Fast>::new(),
				Marker::<Slow>::new(),
			))
			.id();

		app.add_systems(Update, stop_lazy(behavior, agent));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false, false),
			(
				agent.contains::<SimpleMovement>(),
				agent.contains::<Marker<Fast>>(),
				agent.contains::<Marker<Slow>>()
			)
		);
	}
}
