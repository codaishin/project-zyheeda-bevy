use crate::{
	components::config::{Config, SpeedIndex},
	traits::movement_update::MovementUpdate,
};
use bevy::{ecs::query::QueryFilter, prelude::*};
use common::{
	tools::{Done, speed::Speed},
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::ZyheedaCommands,
};

impl<T> AdvanceMovement for T where T: QueryFilter {}

pub(crate) trait AdvanceMovement: QueryFilter + Sized {
	fn advance<TMovement>(
		mut commands: ZyheedaCommands,
		mut agents: Query<(Entity, TMovement::TComponents, &Config, &SpeedIndex), Self>,
	) where
		TMovement: MovementUpdate,
	{
		for (entity, components, config, speed_index) in &mut agents {
			commands.try_apply_on(&entity, |mut e| {
				let e = &mut e;

				if let Done(false) = TMovement::update(e, components, Speed(config[*speed_index])) {
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
		traits::handles_movement::{MovementSpeed, SpeedToggle},
		zyheeda_commands::ZyheedaEntityCommands,
	};
	use testing::SingleThreadedApp;

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
		app.add_systems(Update, Without::<_DoNotMove>::advance::<_Movement>);

		app
	}

	#[test]
	fn apply_fixed_speed() {
		let mut app = setup();
		let agent = Config {
			speed: MovementSpeed::Fixed(UnitsPerSecond::from(11.)),
			..default()
		};
		let movement = _Movement(false.into());
		let agent = app.world_mut().spawn((agent, movement)).id();

		app.update();

		assert_eq!(
			Some(&_Speed(Speed(UnitsPerSecond::from(11.)),)),
			app.world().entity(agent).get::<_Speed>()
		);
	}

	#[test]
	fn apply_dynamic_default_speed() {
		let mut app = setup();
		let agent = Config {
			speed: MovementSpeed::Variable([UnitsPerSecond::from(22.), UnitsPerSecond::from(33.)]),
			..default()
		};
		let movement = _Movement(false.into());
		let agent = app.world_mut().spawn((agent, movement)).id();

		app.update();

		assert_eq!(
			Some(&_Speed(Speed(UnitsPerSecond::from(22.)),)),
			app.world().entity(agent).get::<_Speed>()
		);
	}

	#[test]
	fn apply_dynamic_toggled_speed() {
		let mut app = setup();
		let agent = Config {
			speed: MovementSpeed::Variable([UnitsPerSecond::from(22.), UnitsPerSecond::from(33.)]),
			..default()
		};
		let movement = _Movement(false.into());
		let agent = app
			.world_mut()
			.spawn((agent, movement, SpeedIndex(SpeedToggle::Right)))
			.id();

		app.update();

		assert_eq!(
			Some(&_Speed(Speed(UnitsPerSecond::from(33.)),)),
			app.world().entity(agent).get::<_Speed>()
		);
	}

	#[test]
	fn use_stop_when_done() {
		let mut app = setup();
		let agent = Config::default();
		let movement = _Movement(Done::from(true));
		let agent = app.world_mut().spawn((agent, movement)).id();

		app.update();

		assert_eq!(Some(&_Stop), app.world().entity(agent).get::<_Stop>());
	}

	#[test]
	fn keep_movement_when_not_done() {
		let mut app = setup();
		let agent = Config::default();
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
			.spawn((Config::default(), _Movement(Done::from(false)), _DoNotMove))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_Speed>());
	}
}
