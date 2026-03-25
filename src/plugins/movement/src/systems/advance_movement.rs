use crate::traits::movement_update::MovementUpdate;
use bevy::{ecs::query::QueryFilter, prelude::*};
use common::{
	tools::{Done, speed::Speed},
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::ZyheedaCommands,
};

impl<T> AdvanceMovement for T where T: QueryFilter {}

pub(crate) trait AdvanceMovement: QueryFilter + Sized {
	fn advance<TMovement, TConfig>(
		mut commands: ZyheedaCommands,
		mut agents: Query<(Entity, TMovement::TComponents, &TConfig), Self>,
	) where
		TMovement: MovementUpdate,
		TConfig: Component,
		for<'a> &'a TConfig: Into<Speed>,
	{
		for (entity, components, config) in &mut agents {
			commands.try_apply_on(&entity, |mut e| {
				let e = &mut e;

				if let Done(false) = TMovement::update(e, components, config.into()) {
					return;
				};

				TMovement::stop(e)
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::{Done, UnitsPerSecond, speed::Speed},
		zyheeda_commands::ZyheedaEntityCommands,
	};
	use testing::SingleThreadedApp;

	#[derive(Component, Default)]
	struct _Config(Speed);

	impl From<&'_ _Config> for Speed {
		fn from(_Config(speed): &'_ _Config) -> Self {
			*speed
		}
	}

	#[derive(Component, PartialEq, Debug, Clone, Copy)]
	struct _Stop;

	#[derive(Component, PartialEq, Debug)]
	struct _Speed(Speed);

	#[derive(Component, Default, Debug, PartialEq)]
	struct _Movement(Done);

	impl MovementUpdate for _Movement {
		type TComponents = &'static _Movement;

		fn update(agent: &mut ZyheedaEntityCommands, movement: &_Movement, speed: Speed) -> Done {
			agent.try_insert(_Speed(speed));
			movement.0
		}

		fn stop(entity: &mut ZyheedaEntityCommands) {
			entity.try_insert(_Stop);
		}
	}

	#[derive(Component)]
	struct _DoNotMove;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, Without::<_DoNotMove>::advance::<_Movement, _Config>);

		app
	}

	#[test]
	fn apply_speed() {
		let mut app = setup();
		let agent = _Config(Speed(UnitsPerSecond::from(11.)));
		let movement = _Movement(false.into());
		let agent = app.world_mut().spawn((agent, movement)).id();

		app.update();

		assert_eq!(
			Some(&_Speed(Speed(UnitsPerSecond::from(11.)),)),
			app.world().entity(agent).get::<_Speed>()
		);
	}

	#[test]
	fn use_stop_when_done() {
		let mut app = setup();
		let agent = _Config::default();
		let movement = _Movement(Done::from(true));
		let agent = app.world_mut().spawn((agent, movement)).id();

		app.update();

		assert_eq!(Some(&_Stop), app.world().entity(agent).get::<_Stop>());
	}

	#[test]
	fn keep_movement_when_not_done() {
		let mut app = setup();
		let agent = _Config::default();
		let movement = _Movement(Done::from(false));
		let agent = app.world_mut().spawn((agent, movement)).id();

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
			.spawn((_Config::default(), _Movement(Done::from(false)), _DoNotMove))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_Speed>());
	}
}
