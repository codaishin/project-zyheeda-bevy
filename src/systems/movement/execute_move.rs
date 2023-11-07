use crate::{
	behaviors::MovementMode,
	components::{Idle, Run, Walk},
	traits::{movement::Movement, movement_data::MovementData},
};
use bevy::prelude::*;

pub fn execute_move<
	TAgent: Component + MovementData,
	TMovement: Component + Movement,
	TBehavior: Send + Sync + 'static,
>(
	time: Res<Time>,
	mut commands: Commands,
	mut agents: Query<(Entity, &mut TMovement, &mut Transform, &TAgent)>,
) {
	for (entity, mut movement, mut transform, agent) in agents.iter_mut() {
		let mut entity = commands.entity(entity);
		let (speed, movement_mode) = agent.get_movement_data();
		let is_done = movement.update(&mut transform, time.delta_seconds() * speed.to_f32());

		match (is_done, movement_mode) {
			(true, _) => {
				entity.insert(Idle::<TBehavior>::new());
				entity.remove::<Run>();
				entity.remove::<Walk>();
			}
			(_, MovementMode::Walk) => {
				entity.remove::<Run>();
				entity.insert(Walk);
			}
			(_, MovementMode::Run) => {
				entity.remove::<Walk>();
				entity.insert(Run);
			}
		}
	}
}

#[cfg(test)]
mod move_player_tests {
	use super::*;
	use crate::{
		behaviors::MovementMode,
		components::UnitsPerSecond,
		traits::movement::{IsDone, Movement, Units},
	};
	use mockall::{automock, predicate::eq};
	use std::time::Duration;

	struct MockBehavior;

	#[derive(Component)]
	struct Runner;
	#[derive(Component)]
	struct Walker;

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

	impl MovementData for Runner {
		fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode) {
			(UnitsPerSecond::new(11.), MovementMode::Run)
		}
	}

	impl MovementData for Walker {
		fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode) {
			(UnitsPerSecond::new(1.), MovementMode::Walk)
		}
	}

	fn setup_app() -> App {
		let mut app = App::new();
		let mut time = Time::default();

		time.update();
		app.insert_resource(time);
		app.update();
		app.add_systems(
			Update,
			(
				execute_move::<Runner, _Movement, MockBehavior>,
				execute_move::<Walker, _Movement, MockBehavior>,
			),
		);

		app
	}

	#[test]
	fn move_agent_once() {
		let mut app = setup_app();
		let mut time = app.world.resource_mut::<Time>();

		let last_update = time.last_update().unwrap();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = Runner;
		let time_delta = Duration::from_millis(30);
		let mut movement = _Movement::new();

		movement
			.mock
			.expect_update()
			.with(eq(transform), eq(time_delta.as_secs_f32() * 11.))
			.times(1)
			.return_const(false);

		time.update_with_instant(last_update + time_delta);
		app.world.spawn((agent, movement, transform));

		app.update();
	}

	#[test]
	fn move_agent_twice() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = Runner;
		let mut movement = _Movement::new();

		movement.mock.expect_update().times(2).return_const(false);

		app.world.spawn((agent, movement, transform));

		app.update();
		app.update();
	}

	#[test]
	fn add_idle_when_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = Runner;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(true);

		let agent = app.world.spawn((agent, movement, transform)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Idle<MockBehavior>>());
	}

	#[test]
	fn do_not_add_idle_when_not_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = Runner;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(false);

		let agent = app.world.spawn((agent, movement, transform)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Idle<MockBehavior>>());
	}

	#[test]
	fn set_run_and_remove_walk_component() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = Runner;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(false);

		let agent = app.world.spawn((agent, movement, transform, Walk)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, true),
			(agent.contains::<Walk>(), agent.contains::<Run>())
		)
	}

	#[test]
	fn remove_run_when_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = Runner;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(true);

		let agent = app.world.spawn((agent, movement, transform, Walk)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false),
			(agent.contains::<Walk>(), agent.contains::<Run>())
		)
	}

	#[test]
	fn set_walk_and_remove_run_component() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = Walker;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(false);

		let agent = app.world.spawn((agent, movement, transform, Run)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(true, false),
			(agent.contains::<Walk>(), agent.contains::<Run>())
		)
	}

	#[test]
	fn remove_walk_when_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = Walker;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(true);

		let agent = app.world.spawn((agent, movement, transform, Run)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false),
			(agent.contains::<Walk>(), agent.contains::<Run>())
		)
	}
}
