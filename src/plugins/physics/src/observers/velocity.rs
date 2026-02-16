use crate::components::velocity::LinearVelocity;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl LinearVelocity {
	pub(crate) fn apply(
		on_insert: On<Insert, Self>,
		mut commands: ZyheedaCommands,
		velocities: Query<&Self>,
	) {
		let Ok(Self(velocity)) = velocities.get(on_insert.entity) else {
			return;
		};

		commands.try_apply_on(&on_insert.entity, |mut e| {
			e.try_insert(Velocity::linear(*velocity));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(LinearVelocity::apply);

		app
	}

	#[test]
	fn insert_velocity() {
		let mut app = setup();

		let entity = app.world_mut().spawn(LinearVelocity(Vec3::new(1., 2., 3.)));

		assert_eq!(
			Some(&Velocity::linear(Vec3::new(1., 2., 3.))),
			entity.get::<Velocity>(),
		);
	}

	#[test]
	fn re_insert_velocity() {
		let mut app = setup();
		let mut entity = app.world_mut().spawn(LinearVelocity(Vec3::new(1., 2., 3.)));

		entity.insert(LinearVelocity(Vec3::new(3., 2., 1.)));

		assert_eq!(
			Some(&Velocity::linear(Vec3::new(3., 2., 1.))),
			entity.get::<Velocity>(),
		);
	}
}
