use crate::{
	systems::idle::SetToIdle,
	traits::{IsDone, MovementVelocityBased},
};
use bevy::prelude::*;
use common::{components::Immobilized, tools::speed::Speed, traits::accessors::get::Getter};

impl<T> ExecuteMovement for T {}

pub(crate) trait ExecuteMovement {
	fn execute_movement<TMovement>(
		mut commands: Commands,
		agents: Query<(Entity, &GlobalTransform, &Self, &TMovement), Without<Immobilized>>,
	) -> SetToIdle<TMovement>
	where
		Self: Component + Sized + Getter<Speed>,
		TMovement: Component + MovementVelocityBased,
	{
		let done_entities = agents
			.iter()
			.filter_map(|(id, transform, config, movement)| {
				let Speed(speed) = config.get();
				let entity = &mut commands.get_entity(id)?;
				let position = transform.translation();

				match movement.update(entity, position, speed) {
					IsDone(true) => Some(entity.id()),
					IsDone(false) => None,
				}
			})
			.collect();

		SetToIdle::new(done_entities)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::IsDone;
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::UnitsPerSecond,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	#[derive(Component, Default)]
	struct _Agent(Speed);

	impl Getter<Speed> for _Agent {
		fn get(&self) -> Speed {
			self.0
		}
	}

	#[derive(Component, PartialEq, Debug)]
	struct _MoveParams((Vec3, UnitsPerSecond));

	#[derive(Component, Default, Debug)]
	struct _Movement(IsDone);

	#[derive(Component, Debug, PartialEq)]
	struct _Idle;

	impl MovementVelocityBased for _Movement {
		fn update(
			&self,
			agent: &mut EntityCommands,
			position: Vec3,
			speed: UnitsPerSecond,
		) -> IsDone {
			agent.insert(_MoveParams((position, speed)));
			self.0
		}
	}

	fn idle(set_to_idle: In<SetToIdle<_Movement>>, mut commands: Commands) {
		for entity in set_to_idle.entities.iter() {
			commands.entity(*entity).insert(_Idle);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _Agent::execute_movement::<_Movement>.pipe(idle));

		app
	}

	#[test]
	fn apply_speed() {
		let mut app = setup();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = _Agent(UnitsPerSecond::new(11.).into());
		let movement = _Movement(false.into());
		let agent = app
			.world_mut()
			.spawn((agent, GlobalTransform::from(transform), movement))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&_MoveParams((
				Vec3::new(1., 2., 3.),
				UnitsPerSecond::new(11.)
			))),
			agent.get::<_MoveParams>()
		);
	}

	#[test]
	fn return_entity_to_idle_when_done() {
		let mut app = setup();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = _Agent::default();
		let movement = _Movement(true.into());

		let agent = app
			.world_mut()
			.spawn((agent, GlobalTransform::from(transform), movement))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Idle), agent.get::<_Idle>());
	}

	#[test]
	fn do_not_return_entity_to_idle_when_not_done() {
		let mut app = setup();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = _Agent::default();
		let movement = _Movement(false.into());

		let agent = app
			.world_mut()
			.spawn((agent, GlobalTransform::from(transform), movement))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_Idle>());
	}

	#[test]
	fn no_movement_when_immobilized() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Agent::default(),
				GlobalTransform::from_xyz(1., 2., 3.),
				_Movement(false.into()),
				Immobilized,
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_MoveParams>());
	}
}
