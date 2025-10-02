use crate::traits::MovementUpdate;
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl<T> ExecuteMovement for T where T: Component + MovementUpdate {}

pub(crate) trait ExecuteMovement: Component + MovementUpdate + Sized {
	fn execute_movement(
		mut commands: ZyheedaCommands,
		mut agents: Query<(Entity, Self::TComponents<'_>, &Self), Self::TConstraint>,
	) {
		for (entity, components, movement) in &mut agents {
			commands.try_apply_on(&entity, |mut e| {
				if !movement.update(&mut e, components).is_done() {
					return;
				};

				e.try_remove::<Self>();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{tools::Done, zyheeda_commands::ZyheedaEntityCommands};
	use testing::SingleThreadedApp;

	#[derive(Component, PartialEq, Debug, Clone, Copy)]
	struct _Component;

	#[derive(Component, PartialEq, Debug)]
	struct _MoveParams(_Component);

	#[derive(Component, Default, Debug, PartialEq)]
	struct _Movement(Done);

	impl MovementUpdate for _Movement {
		type TComponents<'a> = &'a _Component;
		type TConstraint = Without<_DoNotMove>;

		fn update(&self, agent: &mut ZyheedaEntityCommands, component: &_Component) -> Done {
			agent.try_insert(_MoveParams(*component));
			self.0
		}
	}

	#[derive(Component)]
	struct _DoNotMove;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _Movement::execute_movement);

		app
	}

	#[test]
	fn apply_component() {
		let mut app = setup();
		let movement = _Movement(false.into());
		let agent = app.world_mut().spawn((_Component, movement)).id();

		app.update();

		assert_eq!(
			Some(&_MoveParams(_Component)),
			app.world().entity(agent).get::<_MoveParams>()
		);
	}

	#[test]
	fn remove_movement_when_done() {
		let mut app = setup();
		let movement = _Movement(Done::from(true));
		let agent = app.world_mut().spawn((_Component, movement)).id();

		app.update();

		assert_eq!(None, app.world().entity(agent).get::<_Movement>());
	}

	#[test]
	fn keep_movement_when_not_done() {
		let mut app = setup();
		let movement = _Movement(Done::from(false));
		let agent = app.world_mut().spawn((_Component, movement)).id();

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
			.spawn((_Component, _Movement(Done::from(false)), _DoNotMove))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_MoveParams>());
	}
}
