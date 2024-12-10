use crate::{
	components::{Movement, VelocityBased},
	traits::{IsDone, MovementVelocityBased},
};
use bevy::{ecs::system::EntityCommands, math::Vec3};
use bevy_rapier3d::dynamics::Velocity;
use common::traits::handles_behaviors::Speed;

const SENSITIVITY: f32 = 0.1;

impl MovementVelocityBased for Movement<VelocityBased> {
	fn update(&self, agent: &mut EntityCommands, position: Vec3, Speed(speed): Speed) -> IsDone {
		let speed = *speed;
		let direction = self.target - position;

		if direction.length() < SENSITIVITY * speed {
			agent.try_insert(Velocity::default());
			return IsDone(true);
		}

		agent.try_insert(Velocity::linear(direction.normalize() * speed));
		IsDone(false)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::{
			component::Component,
			entity::Entity,
			system::{Commands, Query},
		},
	};
	use bevy_rapier3d::dynamics::Velocity;
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::UnitsPerSecond,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	#[derive(Component, Debug, PartialEq)]
	struct _Result(IsDone);

	#[derive(Component)]
	struct _Params((Vec3, Speed));

	fn execute(
		mut commands: Commands,
		agents: Query<(Entity, &Movement<VelocityBased>, &_Params)>,
	) {
		for (id, movement, params) in &agents {
			let agent = &mut commands.entity(id);
			let (position, speed) = params.0;
			let result = movement.update(agent, position, speed);
			agent.insert(_Result(result));
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, execute);

		app
	}

	#[test]
	fn apply_velocity() {
		let mut app = setup();
		let position = Vec3::new(3., 0., 2.);
		let target = Vec3::new(10., 0., 7.);
		let speed = UnitsPerSecond::new(11.);
		let agent = app
			.world_mut()
			.spawn((
				Movement::<VelocityBased>::to(target),
				_Params((position, Speed(speed))),
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);
		let direction = *speed * (target - position).normalize();

		assert_eq!(Some(&Velocity::linear(direction)), agent.get::<Velocity>());
	}

	#[test]
	fn not_done() {
		let mut app = setup();
		let position = Vec3::new(3., 0., 2.);
		let target = Vec3::new(10., 0., 7.);
		let speed = UnitsPerSecond::new(11.);
		let agent = app
			.world_mut()
			.spawn((
				Movement::<VelocityBased>::to(target),
				_Params((position, Speed(speed))),
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Result(IsDone(false))), agent.get::<_Result>());
	}

	#[test]
	fn remove_velocity_when_direction_length_zero() {
		let mut app = setup();
		let position = Vec3::new(10., 0., 7.);
		let target = Vec3::new(10., 0., 7.);
		let speed = UnitsPerSecond::new(11.);
		let agent = app
			.world_mut()
			.spawn((
				Movement::<VelocityBased>::to(target),
				_Params((position, Speed(speed))),
				Velocity::default(),
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&Velocity::default()), agent.get::<Velocity>());
	}

	#[test]
	fn done_when_direction_length_zero() {
		let mut app = setup();
		let position = Vec3::new(10., 0., 7.);
		let target = Vec3::new(10., 0., 7.);
		let speed = UnitsPerSecond::new(11.);
		let agent = app
			.world_mut()
			.spawn((
				Movement::<VelocityBased>::to(target),
				_Params((position, Speed(speed))),
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Result(IsDone(true))), agent.get::<_Result>());
	}

	#[test]
	fn remove_velocity_when_direction_within_sensitivity() {
		let mut app = setup();
		let position = Vec3::new(10., 0., 7.);
		let target = position + Vec3::X * SENSITIVITY * 10.;
		let speed = UnitsPerSecond::new(11.);
		let agent = app
			.world_mut()
			.spawn((
				Movement::<VelocityBased>::to(target),
				_Params((position, Speed(speed))),
				Velocity::default(),
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			(Some(&Velocity::default()), Some(&_Result(IsDone(true)))),
			(agent.get::<Velocity>(), agent.get::<_Result>())
		);
	}
}
