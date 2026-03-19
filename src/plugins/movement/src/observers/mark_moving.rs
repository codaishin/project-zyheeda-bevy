use crate::components::ongoing_movement::{IsMoving, OngoingMovement};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl IsMoving {
	pub(crate) fn mark(
		on_insert: On<Insert, OngoingMovement>,
		mut commands: ZyheedaCommands,
		movements: Query<&OngoingMovement>,
	) {
		let Ok(movement) = movements.get(on_insert.entity) else {
			return;
		};

		commands.try_apply_on(&on_insert.entity, |mut e| {
			match movement {
				OngoingMovement::Stopped => e.try_remove::<IsMoving>(),
				OngoingMovement::Target(..) => e.try_insert(IsMoving),
			};
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_movement::MovementTarget;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(IsMoving::mark);

		app
	}

	#[test_case(Vec3::default(); "position")]
	#[test_case(Dir3::X; "direction")]
	fn add_when_moving<T>(target: T)
	where
		T: Into<MovementTarget>,
	{
		let mut app = setup();

		let entity = app.world_mut().spawn(OngoingMovement::target(target));

		assert_eq!(Some(&IsMoving), entity.get::<IsMoving>());
	}

	#[test_case(Vec3::default(); "position")]
	#[test_case(Dir3::X; "direction")]
	fn add_on_reinsert<T>(target: T)
	where
		T: Into<MovementTarget>,
	{
		let mut app = setup();

		let mut entity = app.world_mut().spawn(OngoingMovement::Stopped);
		entity.insert(OngoingMovement::target(target));

		assert_eq!(Some(&IsMoving), entity.get::<IsMoving>());
	}

	#[test]
	fn remove_when_stopped() {
		let mut app = setup();

		let entity = app.world_mut().spawn(OngoingMovement::Stopped);

		assert_eq!(None, entity.get::<IsMoving>());
	}

	#[test]
	fn remove_on_reinsert() {
		let mut app = setup();

		let mut entity = app
			.world_mut()
			.spawn(OngoingMovement::target(Vec3::default()));
		entity.insert(OngoingMovement::Stopped);

		assert_eq!(None, entity.get::<IsMoving>());
	}
}
