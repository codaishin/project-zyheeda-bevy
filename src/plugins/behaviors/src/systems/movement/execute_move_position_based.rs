use crate::{
	components::Immobilized,
	systems::idle::SetToIdle,
	traits::{MovementData, MovementPositionBased},
};
use bevy::prelude::*;
use common::{tools::Units, traits::clamp_zero_positive::ClampZeroPositive};
use std::ops::Deref;

type Components<'a, TConfig, TMovement> =
	(Entity, &'a mut TMovement, &'a mut Transform, &'a TConfig);

pub(crate) fn execute_move_position_based<
	TMovementConfig: Component + MovementData,
	TMovement: Component + MovementPositionBased,
	TTime: Send + Sync + Default + 'static,
>(
	time: Res<Time<TTime>>,
	mut agents: Query<Components<TMovementConfig, TMovement>, Without<Immobilized>>,
) -> SetToIdle<TMovement> {
	let done_entities = agents
		.iter_mut()
		.filter_map(|(entity, mut movement, mut transform, config)| {
			let (speed, ..) = config.get_movement_data();
			let distance = time.delta_seconds() * *speed.deref();

			match movement
				.update(&mut transform, Units::new(distance))
				.is_done()
			{
				true => Some(entity),
				false => None,
			}
		})
		.collect();

	SetToIdle::new(done_entities)
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{
		components::{Immobilized, MovementMode},
		traits::{IsDone, MovementData},
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::{Units, UnitsPerSecond},
		traits::clamp_zero_positive::ClampZeroPositive,
	};
	use mockall::{automock, predicate::eq};
	use std::time::Duration;

	#[derive(Component)]
	struct ConfigFast;

	#[derive(Component)]
	struct ConfigSlow;

	#[derive(Component, Default, Debug)]
	struct _Movement {
		pub mock: Mock_Movement,
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Idle;

	#[automock]
	impl MovementPositionBased for _Movement {
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

	fn idle(set_to_idle: In<SetToIdle<_Movement>>, mut commands: Commands) {
		for entity in set_to_idle.entities.iter() {
			commands.entity(*entity).insert(_Idle);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		let mut time = Time::<Real>::default();

		time.update();
		app.insert_resource(time);
		app.update();
		app.add_systems(
			Update,
			(
				execute_move_position_based::<ConfigFast, _Movement, Real>.pipe(idle),
				execute_move_position_based::<ConfigSlow, _Movement, Real>.pipe(idle),
			),
		);

		app
	}

	#[test]
	fn move_agent_once() {
		let mut app = setup();
		let mut time = app.world.resource_mut::<Time<Real>>();

		let last_update = time.last_update().unwrap();
		let transform = Transform::from_xyz(1., 2., 3.);
		let config = ConfigFast;
		let time_delta = Duration::from_millis(30);
		let mut movement = _Movement::default();

		movement
			.mock
			.expect_update()
			.with(
				eq(transform),
				eq(Units::new(time_delta.as_secs_f32() * 11.)),
			)
			.times(1)
			.return_const(false);

		time.update_with_instant(last_update + time_delta);
		app.world.spawn((config, movement, transform));

		app.update();
	}

	#[test]
	fn move_agent_twice() {
		let mut app = setup();
		let transform = Transform::from_xyz(1., 2., 3.);
		let config = ConfigFast;
		let mut movement = _Movement::default();

		movement.mock.expect_update().times(2).return_const(false);

		app.world.spawn((config, movement, transform));

		app.update();
		app.update();
	}

	#[test]
	fn return_entity_to_idle_when_done() {
		let mut app = setup();
		let transform = Transform::from_xyz(1., 2., 3.);
		let config = ConfigFast;
		let mut movement = _Movement::default();

		movement.mock.expect_update().return_const(true);

		let agent = app.world.spawn((config, movement, transform)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&_Idle), agent.get::<_Idle>());
	}

	#[test]
	fn do_not_return_entity_to_idle_when_not_done() {
		let mut app = setup();
		let transform = Transform::from_xyz(1., 2., 3.);
		let config = ConfigFast;
		let mut movement = _Movement::default();

		movement.mock.expect_update().return_const(false);

		let agent = app.world.spawn((config, movement, transform)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<_Idle>());
	}

	#[test]
	fn do_not_move_agent_when_immobilized() {
		let mut app = setup();
		let mut time = app.world.resource_mut::<Time<Real>>();

		let last_update = time.last_update().unwrap();
		let mut movement = _Movement::default();

		movement.mock.expect_update().never().return_const(false);

		time.update_with_instant(last_update + Duration::from_millis(30));
		app.world
			.spawn((ConfigFast, movement, Transform::default(), Immobilized));

		app.update();
	}
}
