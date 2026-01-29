use crate::components::motion::Motion;
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl Motion {
	pub(crate) fn zero_velocity_on_remove(
		on_remove: On<Remove, Self>,
		mut commands: ZyheedaCommands,
	) {
		commands.try_apply_on(&on_remove.entity, |mut e| {
			e.try_insert(Velocity::zero());
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy_rapier3d::prelude::Velocity;
	use common::{tools::speed::Speed, traits::handles_physics::LinearMotion};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(Motion::zero_velocity_on_remove);

		app
	}

	#[test]
	fn set_velocity_to_zero_on_remove() {
		let mut app = setup();
		let mut entity = app.world_mut().spawn((
			Motion::Done(LinearMotion::ToTarget {
				speed: Speed::default(),
				target: Vec3::default(),
			}),
			Velocity::linear(Vec3::new(1., 2., 3.)),
		));

		entity.remove::<Motion>();

		assert_eq!(Some(&Velocity::zero()), entity.get::<Velocity>());
	}
}
