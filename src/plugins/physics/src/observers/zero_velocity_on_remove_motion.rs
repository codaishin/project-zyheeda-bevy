use crate::components::motion::Motion;
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl Motion {
	pub(crate) fn zero_velocity_on_remove(
		trigger: Trigger<OnRemove, Self>,
		mut commands: ZyheedaCommands,
	) {
		commands.try_apply_on(&trigger.target(), |mut e| {
			e.try_insert(Velocity::zero());
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy_rapier3d::prelude::Velocity;
	use common::{
		tools::speed::Speed,
		traits::{handles_movement_behavior::MotionSpec, handles_physics::LinearMotionSpec},
	};
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
			Motion::Done(LinearMotionSpec(MotionSpec::ToTarget {
				speed: Speed::default(),
				target: Vec3::default(),
			})),
			Velocity::linear(Vec3::new(1., 2., 3.)),
		));

		entity.remove::<Motion>();

		assert_eq!(Some(&Velocity::zero()), entity.get::<Velocity>());
	}
}
