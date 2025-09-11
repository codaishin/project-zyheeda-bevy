use crate::traits::MovementUpdate;
use bevy::prelude::*;
use common::{
	tools::speed::Speed,
	traits::accessors::get::{RefAs, RefInto, TryApplyOn},
	zyheeda_commands::ZyheedaCommands,
};

impl<T> ExecuteMovement for T where T: Component + Sized + for<'a> RefInto<'a, Speed> {}

pub(crate) trait ExecuteMovement: Component + Sized + for<'a> RefInto<'a, Speed> {
	fn execute_movement<TMovement>(
		mut commands: ZyheedaCommands,
		mut agents: Query<
			(Entity, TMovement::TComponents<'_>, &Self, &TMovement),
			TMovement::TConstraint,
		>,
	) where
		TMovement: Component + MovementUpdate,
	{
		for (entity, components, config, movement) in &mut agents {
			commands.try_apply_on(&entity, |mut e| {
				let speed = config.ref_as::<Speed>();

				if !movement.update(&mut e, components, speed).is_done() {
					return;
				};

				e.try_remove::<TMovement>();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::{Done, UnitsPerSecond},
		zyheeda_commands::ZyheedaEntityCommands,
	};
	use testing::SingleThreadedApp;

	#[derive(Component, Default)]
	struct _Agent(Speed);

	impl RefInto<'_, Speed> for _Agent {
		fn ref_into(&self) -> Speed {
			self.0
		}
	}
	#[derive(Component, PartialEq, Debug, Clone, Copy)]
	struct _Component;

	#[derive(Component, PartialEq, Debug)]
	struct _MoveParams((_Component, Speed));

	#[derive(Component, Default, Debug, PartialEq)]
	struct _Movement(Done);

	impl MovementUpdate for _Movement {
		type TComponents<'a> = &'a _Component;
		type TConstraint = Without<_DoNotMove>;

		fn update(
			&self,
			agent: &mut ZyheedaEntityCommands,
			component: &_Component,
			speed: Speed,
		) -> Done {
			agent.try_insert(_MoveParams((*component, speed)));
			self.0
		}
	}

	#[derive(Component)]
	struct _DoNotMove;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _Agent::execute_movement::<_Movement>);

		app
	}

	#[test]
	fn apply_speed() {
		let mut app = setup();
		let agent = _Agent(UnitsPerSecond::from(11.).into());
		let movement = _Movement(false.into());
		let agent = app.world_mut().spawn((agent, _Component, movement)).id();

		app.update();

		assert_eq!(
			Some(&_MoveParams(
				(_Component, Speed(UnitsPerSecond::from(11.)),)
			)),
			app.world().entity(agent).get::<_MoveParams>()
		);
	}

	#[test]
	fn remove_movement_when_done() {
		let mut app = setup();
		let agent = _Agent::default();
		let movement = _Movement(Done::from(true));
		let agent = app.world_mut().spawn((agent, _Component, movement)).id();

		app.update();

		assert_eq!(None, app.world().entity(agent).get::<_Movement>());
	}

	#[test]
	fn keep_movement_when_not_done() {
		let mut app = setup();
		let agent = _Agent::default();
		let movement = _Movement(Done::from(false));
		let agent = app.world_mut().spawn((agent, _Component, movement)).id();

		app.update();

		assert_eq!(
			Some(&_Movement(Done::from(false))),
			app.world().entity(agent).get::<_Movement>()
		);
	}

	#[test]
	fn no_movement_when_constraint_violated() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Agent::default(),
				_Component,
				_Movement(Done::from(false)),
				_DoNotMove,
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_MoveParams>());
	}
}
