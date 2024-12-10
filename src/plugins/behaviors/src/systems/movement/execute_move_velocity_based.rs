use crate::{
	systems::idle::SetToIdle,
	traits::{IsDone, MovementVelocityBased},
};
use bevy::prelude::*;
use common::{
	components::Immobilized,
	traits::{accessors::get::GetterRef, handles_behaviors::Speed},
};

pub(crate) fn execute_move_velocity_based<TAgent, TMovement>(
	mut commands: Commands,
	agents: Query<(Entity, &TAgent, &GlobalTransform, &TMovement), Without<Immobilized>>,
) -> SetToIdle<TMovement>
where
	TAgent: Component + GetterRef<Speed>,
	TMovement: Component + MovementVelocityBased,
{
	let done_entities = agents
		.iter()
		.filter_map(|(entity, agent, transform, movement)| {
			let entity_cmds = &mut commands.get_entity(entity)?;
			let speed = *agent.get();
			let position = transform.translation();

			match movement.update(entity_cmds, position, speed) {
				IsDone(true) => Some(entity),
				IsDone(false) => None,
			}
		})
		.collect();

	SetToIdle::new(done_entities)
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{EntityCommands, RunSystemOnce};
	use common::{
		tools::UnitsPerSecond,
		traits::{clamp_zero_positive::ClampZeroPositive, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::mock;

	#[derive(Component)]
	struct _Agent(Speed);

	impl GetterRef<Speed> for _Agent {
		fn get(&self) -> &Speed {
			let _Agent(speed) = self;
			speed
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Movement {
		mock: Mock_Movement,
	}

	impl MovementVelocityBased for _Movement {
		fn update(&self, agent: &mut EntityCommands, position: Vec3, speed: Speed) -> IsDone {
			self.mock.update(agent, position, speed)
		}
	}

	mock! {
		_Movement {}
		impl MovementVelocityBased for _Movement {
			fn update<'a>(&self, agent: &mut EntityCommands<'a>, position: Vec3, speed: Speed) -> IsDone;
		}
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn call_movement_update_with_translation_and_speed() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent(Speed(UnitsPerSecond::new(11.))),
			_Movement::new().with_mock(|mock| {
				mock.expect_update()
					.withf(|_, p, s| {
						assert_eq!(
							(&Vec3::new(1., 2., 3.), &Speed(UnitsPerSecond::new(11.)),),
							(p, s)
						);
						true
					})
					.times(1)
					.return_const(IsDone(true));
			}),
			GlobalTransform::from_xyz(1., 2., 3.),
		));

		app.world_mut()
			.run_system_once(execute_move_velocity_based::<_Agent, _Movement>);
	}

	#[test]
	fn call_movement_update_with_other_translation_and_speed() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent(Speed(UnitsPerSecond::new(3.))),
			_Movement::new().with_mock(|mock| {
				mock.expect_update()
					.withf(|_, p, s| {
						assert_eq!(
							(&Vec3::new(4., 5., 6.), &Speed(UnitsPerSecond::new(3.)),),
							(p, s)
						);
						true
					})
					.times(1)
					.return_const(IsDone(true));
			}),
			GlobalTransform::from_xyz(4., 5., 6.),
		));

		app.world_mut()
			.run_system_once(execute_move_velocity_based::<_Agent, _Movement>);
	}

	#[test]
	fn return_set_to_idle_with_done_entity() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Agent(Speed(UnitsPerSecond::new(3.))),
				_Movement::new().with_mock(|mock| {
					mock.expect_update().return_const(IsDone(true));
				}),
				GlobalTransform::default(),
			))
			.id();

		let SetToIdle { entities, .. } = app
			.world_mut()
			.run_system_once(execute_move_velocity_based::<_Agent, _Movement>);

		assert_eq!(vec![agent], entities);
	}

	#[test]
	fn return_set_to_idle_without_not_done_entity() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent(Speed(UnitsPerSecond::new(3.))),
			_Movement::new().with_mock(|mock| {
				mock.expect_update().return_const(IsDone(false));
			}),
			GlobalTransform::default(),
		));

		let SetToIdle { entities, .. } = app
			.world_mut()
			.run_system_once(execute_move_velocity_based::<_Agent, _Movement>);

		assert_eq!(vec![] as Vec<Entity>, entities);
	}

	#[test]
	fn no_movement_when_immobilized() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent(Speed(UnitsPerSecond::new(3.))),
			_Movement::new().with_mock(|mock| {
				mock.expect_update().never().return_const(IsDone(true));
			}),
			GlobalTransform::default(),
			Immobilized,
		));

		app.world_mut()
			.run_system_once(execute_move_velocity_based::<_Agent, _Movement>);
	}
}
