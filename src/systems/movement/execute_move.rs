use crate::{
	components::{Animate, WaitNext},
	traits::{movement::Movement, movement_data::MovementData},
};
use bevy::prelude::*;

type Components<'a, TAnimationKey, TAgent, TMovement> = (
	Entity,
	&'a mut TMovement,
	&'a mut Transform,
	&'a TAgent,
	Option<&'a Animate<TAnimationKey>>,
);

pub fn execute_move<
	TAnimationKey: PartialEq + Clone + Copy + Send + Sync + 'static,
	TAgent: Component + MovementData<TAnimationKey>,
	TMovement: Component + Movement,
	TTime: Send + Sync + Default + 'static,
>(
	time: Res<Time<TTime>>,
	mut commands: Commands,
	mut agents: Query<Components<TAnimationKey, TAgent, TMovement>>,
) {
	for (entity, mut movement, mut transform, agent, running_animate) in agents.iter_mut() {
		let mut entity = commands.entity(entity);
		let (speed, animate) = agent.get_movement_data();
		let is_done = movement.update(&mut transform, time.delta_seconds() * speed.to_f32());

		if is_done {
			entity.insert(WaitNext);
			entity.remove::<TMovement>();
			if matches!(running_animate, Some(running_animate) if running_animate == &animate) {
				entity.remove::<Animate<TAnimationKey>>();
			}
		} else {
			entity.insert(animate);
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{
		components::{Animate, UnitsPerSecond},
		traits::movement::{IsDone, Movement, Units},
	};
	use mockall::{automock, predicate::eq};
	use std::time::Duration;

	#[derive(Component)]
	struct AgentRun;

	#[derive(Component)]
	struct AgentWalk;

	#[derive(Clone, Copy, PartialEq, Debug)]
	enum _Key {
		Slow,
		Fast,
	}

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

	impl MovementData<_Key> for AgentRun {
		fn get_movement_data(&self) -> (UnitsPerSecond, Animate<_Key>) {
			(UnitsPerSecond::new(11.), Animate::Repeat(_Key::Fast))
		}
	}

	impl MovementData<_Key> for AgentWalk {
		fn get_movement_data(&self) -> (UnitsPerSecond, Animate<_Key>) {
			(UnitsPerSecond::new(1.), Animate::Repeat(_Key::Slow))
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
				execute_move::<_Key, AgentRun, _Movement, Real>,
				execute_move::<_Key, AgentWalk, _Movement, Real>,
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
	fn set_fast() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = AgentRun;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(false);

		let agent = app.world.spawn((agent, movement, transform)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Animate::Repeat(_Key::Fast)),
			agent.get::<Animate<_Key>>()
		);
	}

	#[test]
	fn remove_fast_when_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = AgentRun;
		let mut movement = _Movement::new();
		let (_, animate) = agent.get_movement_data();

		movement.mock.expect_update().return_const(true);

		let agent = app.world.spawn((agent, movement, transform, animate)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Animate<_Key>>());
	}

	#[test]
	fn set_slow() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = AgentWalk;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(false);

		let agent = app.world.spawn((agent, movement, transform)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Animate::Repeat(_Key::Slow)),
			agent.get::<Animate<_Key>>()
		);
	}

	#[test]
	fn remove_slow_when_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = AgentWalk;
		let mut movement = _Movement::new();
		let (_, animate) = agent.get_movement_data();

		movement.mock.expect_update().return_const(true);

		let agent = app.world.spawn((agent, movement, transform, animate)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Animate<_Key>>());
	}

	#[test]
	fn remove_movement_when_done() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = AgentWalk;
		let mut movement = _Movement::new();

		movement.mock.expect_update().return_const(true);

		let agent = app.world.spawn((agent, movement, transform)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<_Movement>());
	}

	#[test]
	fn do_not_remove_animate_when_not_matching_move_mode() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = AgentWalk;
		let mut movement = _Movement::new();
		let (_, animate) = AgentRun.get_movement_data();

		movement.mock.expect_update().return_const(true);

		let agent = app.world.spawn((agent, movement, transform, animate)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&animate), agent.get::<Animate<_Key>>());
	}
}
