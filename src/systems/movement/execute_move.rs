use crate::{
	behaviors::MovementMode,
	components::{Marker, WaitNext},
	markers::{Fast, Slow},
	traits::{movement::Movement, movement_data::MovementData},
};
use bevy::prelude::*;

pub fn execute_move<TAgent: Component + MovementData, TMovement: Component + Movement>(
	time: Res<Time<Real>>,
	mut commands: Commands,
	mut agents: Query<(Entity, &mut TMovement, &mut Transform, &TAgent)>,
) {
	for (entity, mut movement, mut transform, agent) in agents.iter_mut() {
		let mut entity = commands.entity(entity);
		let (speed, movement_mode) = agent.get_movement_data();
		let is_done = movement.update(&mut transform, time.delta_seconds() * speed.to_f32());

		match (is_done, movement_mode) {
			(true, _) => {
				entity.remove::<(Marker<Fast>, Marker<Slow>, TMovement)>();
				entity.insert(WaitNext);
			}
			(_, MovementMode::Slow) => {
				entity.remove::<Marker<Fast>>();
				entity.insert(Marker::<Slow>::new());
			}
			(_, MovementMode::Fast) => {
				entity.remove::<Marker<Slow>>();
				entity.insert(Marker::<Fast>::new());
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{
		behaviors::MovementMode,
		components::UnitsPerSecond,
		traits::movement::{IsDone, Movement, Units},
	};
	use mockall::{automock, predicate::eq};
	use std::time::Duration;

	#[derive(Component)]
	struct AgentRun;
	#[derive(Component)]
	struct AgentWalk;

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

	impl MovementData for AgentRun {
		fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode) {
			(UnitsPerSecond::new(11.), MovementMode::Fast)
		}
	}

	impl MovementData for AgentWalk {
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
				execute_move::<AgentRun, _Movement>,
				execute_move::<AgentWalk, _Movement>,
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
		let agent = AgentRun;
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
		let agent = AgentRun;
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
		let agent = AgentRun;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(true);

		let agent = app.world.spawn((agent, movement, transform)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<WaitNext>());
	}

	#[test]
	fn do_not_add_idle_when_not_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = AgentRun;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(false);

		let agent = app.world.spawn((agent, movement, transform)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<WaitNext>());
	}

	#[test]
	fn set_run_and_remove_walk_component() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = AgentRun;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(false);

		let agent = app
			.world
			.spawn((agent, movement, transform, Marker::<Slow>::new()))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, true),
			(
				agent.contains::<Marker<Slow>>(),
				agent.contains::<Marker<Fast>>()
			)
		)
	}

	#[test]
	fn remove_run_when_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = AgentRun;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(true);

		let agent = app
			.world
			.spawn((agent, movement, transform, Marker::<Slow>::new()))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false),
			(
				agent.contains::<Marker<Slow>>(),
				agent.contains::<Marker<Fast>>()
			)
		)
	}

	#[test]
	fn set_walk_and_remove_run_component() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = AgentWalk;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(false);

		let agent = app
			.world
			.spawn((agent, movement, transform, Marker::<Fast>::new()))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(true, false),
			(
				agent.contains::<Marker<Slow>>(),
				agent.contains::<Marker<Fast>>()
			)
		)
	}

	#[test]
	fn remove_walk_when_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = AgentWalk;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(true);

		let agent = app
			.world
			.spawn((agent, movement, transform, Marker::<Fast>::new()))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false),
			(
				agent.contains::<Marker<Slow>>(),
				agent.contains::<Marker<Fast>>()
			)
		)
	}

	#[test]
	fn remove_movement_when_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = AgentWalk;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(true);

		let agent = app
			.world
			.spawn((agent, movement, transform, Marker::<Fast>::new()))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<_Movement>());
	}

	#[test]
	fn do_not_remove_movement_when_waiting_next_on_other_agents() {
		#[derive(Component)]
		struct OtherAgent;

		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = OtherAgent;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(false);

		let agent = app
			.world
			.spawn((agent, movement, transform, Marker::<Fast>::new(), WaitNext))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(true, true),
			(
				agent.contains::<_Movement>(),
				agent.contains::<Marker<Fast>>(),
			)
		);
	}
}
