use super::ToLazy;
use crate::components::{
	lazy::Lazy,
	marker::{Fast, Marker, Slow},
	Agent,
	SimpleMovement,
	Spawner,
};
use bevy::{
	ecs::system::Commands,
	math::{Ray, Vec3},
};

fn run_fn(commands: &mut Commands, agent: Agent, _spawner: Spawner, ray: Ray) {
	let Some(length) = ray.intersect_plane(Vec3::ZERO, Vec3::Y) else {
		return;
	};
	commands.entity(agent.0).insert(SimpleMovement {
		target: ray.origin + ray.direction * length,
	});
}

fn stop_fn(commands: &mut Commands, agent: Agent) {
	commands
		.entity(agent.0)
		.remove::<(SimpleMovement, Marker<Fast>, Marker<Slow>)>();
}

impl ToLazy for SimpleMovement {
	fn to_lazy() -> Option<Lazy> {
		Some(Lazy::new(Some(run_fn), Some(stop_fn)))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		test_tools::assert_eq_approx,
		traits::to_lazy::test_tools::{run_lazy, stop_lazy},
	};
	use bevy::prelude::{App, Ray, Update, Vec3};

	#[test]
	fn move_to_zero() {
		let mut app = App::new();
		let lazy = SimpleMovement::to_lazy();
		let agent = Agent(app.world.spawn(()).id());
		let ray = Ray {
			origin: Vec3::Y,
			direction: Vec3::NEG_Y,
		};

		app.add_systems(Update, run_lazy(lazy, agent, Spawner::default(), ray));
		app.update();

		let movement = app.world.entity(agent.0).get::<SimpleMovement>();

		assert_eq!(Some(Vec3::ZERO), movement.map(|m| m.target));
	}

	#[test]
	fn move_to_offset() {
		let mut app = App::new();
		let lazy = SimpleMovement::to_lazy();
		let agent = Agent(app.world.spawn(()).id());
		let ray = Ray {
			origin: Vec3::ONE,
			direction: Vec3::NEG_Y,
		};

		app.add_systems(Update, run_lazy(lazy, agent, Spawner::default(), ray));
		app.update();

		let movement = app.world.entity(agent.0).get::<SimpleMovement>().unwrap();

		assert_eq!(Vec3::new(1., 0., 1.), movement.target);
	}

	#[test]
	fn move_to_offset_2() {
		let mut app = App::new();
		let lazy = SimpleMovement::to_lazy();
		let agent = Agent(app.world.spawn(()).id());
		let ray = Ray {
			origin: Vec3::new(0., 4., 0.),
			direction: Vec3::new(0., -4., 3.),
		};

		app.add_systems(Update, run_lazy(lazy, agent, Spawner::default(), ray));
		app.update();

		let movement = app.world.entity(agent.0).get::<SimpleMovement>().unwrap();

		assert_eq_approx(Vec3::new(0., 0., 3.), movement.target, 0.000001);
	}

	#[test]
	fn remove_all_relevant_components() {
		let mut app = App::new();
		let lazy = SimpleMovement::to_lazy();
		let agent = Agent(
			app.world
				.spawn((
					SimpleMovement { target: Vec3::ZERO },
					Marker::<Fast>::new(),
					Marker::<Slow>::new(),
				))
				.id(),
		);

		app.add_systems(Update, stop_lazy(lazy, agent));
		app.update();

		let agent = app.world.entity(agent.0);

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
