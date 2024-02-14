use crate::{
	components::Idle,
	traits::{Movement, MovementData},
};
use bevy::prelude::*;

type Components<'a, TAgent, TMovement> = (Entity, &'a mut TMovement, &'a mut Transform, &'a TAgent);

pub(crate) fn execute_move<
	TMovementConfig: Component + MovementData,
	TMovement: Component + Movement,
	TTime: Send + Sync + Default + 'static,
>(
	time: Res<Time<TTime>>,
	mut commands: Commands,
	mut agents: Query<Components<TMovementConfig, TMovement>>,
) {
	for (entity, mut movement, mut transform, config) in agents.iter_mut() {
		let mut entity = commands.entity(entity);
		let (speed, ..) = config.get_movement_data();
		let is_done = movement.update(&mut transform, time.delta_seconds() * speed.to_f32());

		if is_done {
			entity.insert(Idle);
			entity.remove::<TMovement>();
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{
		components::MovementMode,
		traits::{IsDone, MovementData, Units},
	};
	use common::tools::UnitsPerSecond;
	use mockall::{automock, predicate::eq};
	use std::time::Duration;

	#[derive(Component)]
	struct ConfigFast;

	#[derive(Component)]
	struct ConfigSlow;

	#[derive(Component)]
	struct _Movement {
		pub mock: Mock_Movement,
	}

	impl _Movement {
		fn new() -> Self {
			Self {
				mock: Mock_Movement::new(),
			}
		}
	}

	#[automock]
	impl Movement for _Movement {
		fn update(&mut self, agent: &mut Transform, distance: Units) -> IsDone {
			self.mock.update(agent, distance)
		}
	}

	impl MovementData for ConfigFast {
		fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode) {
			(UnitsPerSecond::new(11.), MovementMode::Fast)
		}
	}

	impl MovementData for ConfigSlow {
		fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode) {
			(UnitsPerSecond::new(1.), MovementMode::Slow)
		}
	}

	fn setup_app() -> App {
		let mut app = App::new();
		let mut time = Time::<Real>::default();

		time.update();
		app.insert_resource(time);
		app.update();
		app.add_systems(
			Update,
			(
				execute_move::<ConfigFast, _Movement, Real>,
				execute_move::<ConfigSlow, _Movement, Real>,
			),
		);

		app
	}

	#[test]
	fn move_agent_once() {
		let mut app = setup_app();
		let mut time = app.world.resource_mut::<Time<Real>>();

		let last_update = time.last_update().unwrap();
		let transform = Transform::from_xyz(1., 2., 3.);
		let config = ConfigFast;
		let time_delta = Duration::from_millis(30);
		let mut movement = _Movement::new();

		movement
			.mock
			.expect_update()
			.with(eq(transform), eq(time_delta.as_secs_f32() * 11.))
			.times(1)
			.return_const(false);

		time.update_with_instant(last_update + time_delta);
		app.world.spawn((config, movement, transform));

		app.update();
	}

	#[test]
	fn move_agent_twice() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let config = ConfigFast;
		let mut movement = _Movement::new();

		movement.mock.expect_update().times(2).return_const(false);

		app.world.spawn((config, movement, transform));

		app.update();
		app.update();
	}

	#[test]
	fn add_idle_when_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let config = ConfigFast;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(true);

		let agent = app.world.spawn((config, movement, transform)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Idle>());
	}

	#[test]
	fn do_not_add_idle_when_not_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let config = ConfigFast;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(false);

		let agent = app.world.spawn((config, movement, transform)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Idle>());
	}

	#[test]
	fn remove_movement_when_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let config = ConfigSlow;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(true);

		let agent = app.world.spawn((config, movement, transform)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<_Movement>());
	}
}
