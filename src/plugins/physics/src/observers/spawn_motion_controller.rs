use crate::components::motion_controller::{MotionControlParameters, MotionControllerOf};
use bevy::prelude::*;
use common::zyheeda_commands::ZyheedaCommands;

impl MotionControllerOf {
	pub(crate) fn spawn(on_add: On<Add, MotionControlParameters>, mut commands: ZyheedaCommands) {
		commands.spawn(MotionControllerOf(on_add.entity));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::Units,
		traits::handles_physics::physical_bodies::{Shape, ShapeParameters},
	};
	use std::collections::HashSet;
	use testing::{SingleThreadedApp, assert_count};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(MotionControllerOf::spawn);

		app
	}

	fn params() -> MotionControlParameters {
		MotionControlParameters {
			shape: Shape::Parameters(ShapeParameters::Sphere {
				radius: Units::from(11.),
			}),
			blockers: HashSet::new(),
		}
	}

	#[test]
	fn spawn_controller() {
		let mut app = setup();

		let agent = app.world_mut().spawn(params()).id();

		let mut controllers = app.world_mut().query::<&MotionControllerOf>();
		let [controller] = assert_count!(1, controllers.iter(app.world()));
		assert_eq!(&MotionControllerOf(agent), controller);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();

		let agent = app.world_mut().spawn(params()).id();
		let mut controllers = app
			.world_mut()
			.query_filtered::<Entity, With<MotionControllerOf>>();
		let [controller] = assert_count!(1, controllers.iter(app.world()));
		app.world_mut().despawn(controller);
		app.world_mut().entity_mut(agent).insert(params());

		let mut controllers = app
			.world_mut()
			.query_filtered::<(), With<MotionControllerOf>>();
		assert_count!(0, controllers.iter(app.world()));
	}
}
