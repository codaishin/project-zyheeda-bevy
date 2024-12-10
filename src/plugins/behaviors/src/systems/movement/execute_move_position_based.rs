use crate::{
	systems::idle::SetToIdle,
	traits::{IsDone, MovementPositionBased},
};
use bevy::prelude::*;
use common::{
	components::Immobilized,
	tools::Units,
	traits::{
		accessors::get::GetterRef,
		clamp_zero_positive::ClampZeroPositive,
		handles_behaviors::Speed,
	},
};

pub(crate) fn execute_move_position_based<TAgent, TMovement, TTime>(
	time: Res<Time<TTime>>,
	mut agents: Query<(Entity, &TAgent, &mut Transform, &mut TMovement), Without<Immobilized>>,
) -> SetToIdle<TMovement>
where
	TAgent: Component + GetterRef<Speed>,
	TMovement: Component + MovementPositionBased,
	TTime: Send + Sync + Default + 'static,
{
	let done_entities = agents
		.iter_mut()
		.filter_map(|(entity, agent, mut transform, mut movement)| {
			let Speed(speed) = *agent.get();
			let distance = *speed * time.delta_seconds();

			match movement.update(&mut transform, Units::new(distance)) {
				IsDone(true) => Some(entity),
				IsDone(false) => None,
			}
		})
		.collect();

	SetToIdle::new(done_entities)
}

#[cfg(test)]
mod test {
	use super::*;
	use bevy::ecs::system::RunSystemOnce;
	use common::{
		test_tools::utils::TickTime,
		tools::{Units, UnitsPerSecond},
		traits::{clamp_zero_positive::ClampZeroPositive, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::time::Duration;

	#[derive(Component)]
	struct _Agent(Speed);

	impl GetterRef<Speed> for _Agent {
		fn get(&self) -> &Speed {
			let _Agent(speed) = self;
			speed
		}
	}

	#[derive(Component, NestedMocks, Debug)]
	struct _Movement {
		pub mock: Mock_Movement,
	}

	#[automock]
	impl MovementPositionBased for _Movement {
		fn update(&mut self, agent: &mut Transform, distance: Units) -> IsDone {
			self.mock.update(agent, distance)
		}
	}

	fn setup() -> App {
		let mut app = App::new();
		app.init_resource::<Time<Real>>();
		app.tick_time(Duration::default());

		app
	}

	#[test]
	fn move_agent_once() {
		let mut app = setup();
		let delta = Duration::from_millis(30);
		app.tick_time(delta);
		app.world_mut().spawn((
			_Agent(Speed(UnitsPerSecond::new(11.))),
			_Movement::new().with_mock(|mock| {
				mock.expect_update()
					.with(
						eq(Transform::from_xyz(1., 2., 3.)),
						eq(Units::new(delta.as_secs_f32() * 11.)),
					)
					.times(1)
					.return_const(IsDone(false));
			}),
			Transform::from_xyz(1., 2., 3.),
		));

		app.world_mut()
			.run_system_once(execute_move_position_based::<_Agent, _Movement, Real>);
	}

	#[test]
	fn return_set_to_idle_with_done_entity() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Agent(Speed(UnitsPerSecond::new(11.))),
				_Movement::new().with_mock(|mock| {
					mock.expect_update().return_const(IsDone(true));
				}),
				Transform::default(),
			))
			.id();

		let SetToIdle { entities, .. } = app
			.world_mut()
			.run_system_once(execute_move_position_based::<_Agent, _Movement, Real>);

		assert_eq!(vec![agent], entities);
	}

	#[test]
	fn return_set_to_idle_without_not_done_entity() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent(Speed(UnitsPerSecond::new(11.))),
			_Movement::new().with_mock(|mock| {
				mock.expect_update().return_const(IsDone(false));
			}),
			Transform::default(),
		));

		let SetToIdle { entities, .. } = app
			.world_mut()
			.run_system_once(execute_move_position_based::<_Agent, _Movement, Real>);

		assert_eq!(vec![] as Vec<Entity>, entities);
	}

	#[test]
	fn do_not_move_agent_when_immobilized() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent(Speed(UnitsPerSecond::new(11.))),
			_Movement::new().with_mock(|mock| {
				mock.expect_update().never().return_const(IsDone(false));
			}),
			Transform::default(),
			Immobilized,
		));

		app.world_mut()
			.run_system_once(execute_move_position_based::<_Agent, _Movement, Real>);
	}
}
