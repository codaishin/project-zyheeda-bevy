use crate::traits::{IsDone, MovementVelocityBased};
use bevy::prelude::*;
use common::{components::Immobilized, tools::speed::Speed, traits::accessors::get::Getter};

impl<T> ExecuteMovement for T {}

pub(crate) trait ExecuteMovement {
	fn execute_movement<TMovement>(
		mut commands: Commands,
		agents: Query<(Entity, &GlobalTransform, &Self, &TMovement), Without<Immobilized>>,
	) where
		Self: Component + Sized + Getter<Speed>,
		TMovement: Component + MovementVelocityBased,
	{
		for (id, transform, config, movement) in &agents {
			let Some(mut entity) = commands.get_entity(id) else {
				continue;
			};
			let Speed(speed) = config.get();
			let position = transform.translation();

			let IsDone(true) = movement.update(&mut entity, position, speed) else {
				continue;
			};

			entity.remove::<TMovement>();
		}
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

	#[derive(Component, Default, Debug, PartialEq)]
	struct _Movement(IsDone);

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

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _Agent::execute_movement::<_Movement>);

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

		assert_eq!(
			Some(&_MoveParams((
				Vec3::new(1., 2., 3.),
				UnitsPerSecond::new(11.)
			))),
			app.world().entity(agent).get::<_MoveParams>()
		);
	}

	#[test]
	fn remove_movement_when_done() {
		let mut app = setup();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = _Agent::default();
		let movement = _Movement(true.into());
		let agent = app
			.world_mut()
			.spawn((agent, GlobalTransform::from(transform), movement))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(agent).get::<_Movement>());
	}

	#[test]
	fn do_not_return_entity_for_cleanup_when_not_done() {
		let mut app = setup();
		let transform = Transform::from_xyz(1., 2., 3.);
		let agent = _Agent::default();
		let movement = _Movement(false.into());
		let agent = app
			.world_mut()
			.spawn((agent, GlobalTransform::from(transform), movement))
			.id();

		app.update();

		assert_eq!(
			Some(&_Movement(false.into())),
			app.world().entity(agent).get::<_Movement>()
		);
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
