use crate::{
	components::Group,
	traits::{movement::Movement, speed::Speed},
};
use bevy::prelude::*;

#[allow(clippy::type_complexity)]
pub fn execute<
	TAgent: Component + Speed<TMode>,
	TBehavior: Send + Sync + 'static,
	TMovement: Component + Movement,
	TMode: Component,
>(
	time: Res<Time>,
	mut commands: Commands,
	mut agents: Query<
		(Entity, &mut TMovement, &mut Transform, &TAgent),
		(With<TMode>, With<Group<TBehavior>>),
	>,
) {
	for (id, mut movement, mut transform, agent) in agents.iter_mut() {
		let speed = agent.get_speed().to_f32();
		let is_done = movement.update(&mut transform, time.delta_seconds() * speed);
		if is_done {
			commands.entity(id).remove::<Group<TBehavior>>();
		}
	}
}

#[cfg(test)]
mod move_player_tests {
	use super::*;
	use crate::{
		components::UnitsPerSecond,
		traits::movement::{IsDone, Movement, Units},
	};
	use mockall::{automock, predicate::eq};
	use std::time::Duration;

	struct Behavior;

	#[derive(Component)]
	struct Run;

	#[derive(Component)]
	struct Walk;

	#[derive(Component)]
	struct Agent;

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

	impl Speed<Run> for Agent {
		fn get_speed(&self) -> UnitsPerSecond {
			UnitsPerSecond::new(11.)
		}
	}

	impl Speed<Walk> for Agent {
		fn get_speed(&self) -> UnitsPerSecond {
			UnitsPerSecond::new(1.)
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
				execute::<Agent, Behavior, _Movement, Run>,
				execute::<Agent, Behavior, _Movement, Walk>,
			),
		);

		app
	}

	#[test]
	fn move_agent_once_with_run() {
		let mut app = setup_app();
		let mut time = app.world.resource_mut::<Time>();

		let last_update = time.last_update().unwrap();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = Agent;
		let run = Run;
		let group = Group::<Behavior>::new();
		let time_delta = Duration::from_millis(30);
		let mut movement = _Movement::new();

		movement
			.mock
			.expect_update()
			.with(eq(transform), eq(time_delta.as_secs_f32() * 11.))
			.times(1)
			.return_const(false);

		time.update_with_instant(last_update + time_delta);
		app.world.spawn((agent, movement, run, group, transform));

		app.update();
	}

	#[test]
	fn move_agent_once_with_walk() {
		let mut app = setup_app();
		let mut time = app.world.resource_mut::<Time>();

		let last_update = time.last_update().unwrap();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = Agent;
		let walk = Walk;
		let group = Group::<Behavior>::new();
		let time_delta = Duration::from_millis(30);
		let mut movement = _Movement::new();

		movement
			.mock
			.expect_update()
			.with(eq(transform), eq(time_delta.as_secs_f32() * 1.))
			.times(1)
			.return_const(false);

		time.update_with_instant(last_update + time_delta);
		app.world.spawn((agent, movement, walk, group, transform));

		app.update();
	}

	#[test]
	fn do_not_move_without_behavior_group() {
		let mut app = setup_app();
		let mut time = app.world.resource_mut::<Time>();

		let last_update = time.last_update().unwrap();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = Agent;
		let run = Run;
		let time_delta = Duration::from_millis(30);
		let mut movement = _Movement::new();

		movement.mock.expect_update().times(0).return_const(false);

		time.update_with_instant(last_update + time_delta);
		app.world.spawn((agent, movement, run, transform));

		app.update();
	}

	#[test]
	fn move_agent_twice() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = Agent;
		let run = Run;
		let group = Group::<Behavior>::new();
		let mut movement = _Movement::new();

		movement.mock.expect_update().times(2).return_const(false);

		app.world.spawn((agent, movement, run, group, transform));

		app.update();
		app.update();
	}

	#[test]
	fn remove_busy_when_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = Agent;
		let run = Run;
		let group = Group::<Behavior>::new();
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(true);

		let agent = app
			.world
			.spawn((agent, movement, run, group, transform))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Group<Behavior>>());
	}

	#[test]
	fn do_not_remove_busy_when_not_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = Agent;
		let run = Run;
		let group = Group::<Behavior>::new();
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(false);

		let agent = app
			.world
			.spawn((group, agent, movement, run, transform))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Group<Behavior>>());
	}
}
