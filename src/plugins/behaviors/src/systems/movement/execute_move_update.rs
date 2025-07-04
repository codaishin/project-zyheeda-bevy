use crate::traits::{IsDone, MovementUpdate};
use bevy::prelude::*;
use common::{
	tools::speed::Speed,
	traits::accessors::get::{GetField, Getter},
};
use std::time::Duration;

impl<T> ExecuteMovement for T {}

pub(crate) trait ExecuteMovement {
	fn execute_movement<TMovement>(
		In(delta): In<Duration>,
		mut commands: Commands,
		mut agents: Query<
			(Entity, TMovement::TComponents<'_>, &Self, &TMovement),
			TMovement::TConstraint,
		>,
	) where
		Self: Component + Sized + Getter<Speed>,
		TMovement: Component + MovementUpdate,
	{
		for (id, components, config, movement) in &mut agents {
			let Ok(mut entity) = commands.get_entity(id) else {
				continue;
			};
			let speed = Speed::get_field(config);

			let IsDone(true) = movement.update(&mut entity, components, speed, delta) else {
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
	use common::{tools::UnitsPerSecond, traits::clamp_zero_positive::ClampZeroPositive};
	use testing::SingleThreadedApp;

	#[derive(Component, Default)]
	struct _Agent(Speed);

	impl Getter<Speed> for _Agent {
		fn get(&self) -> Speed {
			self.0
		}
	}
	#[derive(Component, PartialEq, Debug, Clone, Copy)]
	struct _Component;

	#[derive(Component, PartialEq, Debug)]
	struct _MoveParams((_Component, Speed, Duration));

	#[derive(Component, Default, Debug, PartialEq)]
	struct _Movement(IsDone);

	impl MovementUpdate for _Movement {
		type TComponents<'a> = &'a _Component;
		type TConstraint = Without<_DoNotMove>;

		fn update(
			&self,
			agent: &mut EntityCommands,
			component: &_Component,
			speed: Speed,
			delta: Duration,
		) -> IsDone {
			agent.insert(_MoveParams((*component, speed, delta)));
			self.0
		}
	}

	#[derive(Component)]
	struct _DoNotMove;

	fn setup(delta: Duration) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			(move || delta).pipe(_Agent::execute_movement::<_Movement>),
		);

		app
	}

	#[test]
	fn apply_speed_and_delta() {
		let mut app = setup(Duration::from_millis(100));
		let agent = _Agent(UnitsPerSecond::new(11.).into());
		let movement = _Movement(false.into());
		let agent = app.world_mut().spawn((agent, _Component, movement)).id();

		app.update();

		assert_eq!(
			Some(&_MoveParams((
				_Component,
				Speed(UnitsPerSecond::new(11.)),
				Duration::from_millis(100)
			))),
			app.world().entity(agent).get::<_MoveParams>()
		);
	}

	#[test]
	fn remove_movement_when_done() {
		let mut app = setup(Duration::default());
		let agent = _Agent::default();
		let movement = _Movement(IsDone(true));
		let agent = app.world_mut().spawn((agent, _Component, movement)).id();

		app.update();

		assert_eq!(None, app.world().entity(agent).get::<_Movement>());
	}

	#[test]
	fn do_not_return_entity_for_cleanup_when_not_done() {
		let mut app = setup(Duration::default());
		let agent = _Agent::default();
		let movement = _Movement(IsDone(false));
		let agent = app.world_mut().spawn((agent, _Component, movement)).id();

		app.update();

		assert_eq!(
			Some(&_Movement(IsDone(false))),
			app.world().entity(agent).get::<_Movement>()
		);
	}

	#[test]
	fn no_movement_when_constraint_violated() {
		let mut app = setup(Duration::default());
		let agent = app
			.world_mut()
			.spawn((
				_Agent::default(),
				_Component,
				_Movement(IsDone(false)),
				_DoNotMove,
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_MoveParams>());
	}
}
