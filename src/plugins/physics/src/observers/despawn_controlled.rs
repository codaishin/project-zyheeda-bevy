use crate::components::{motion_controller::MotionControllerOf, skill::Skill};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl MotionControllerOf<Skill> {
	pub(crate) fn despawn_controlled(
		on_despawn: On<Despawn<Self>>,
		controllers: Query<&Self>,
		mut commands: ZyheedaCommands,
	) {
		let Ok(controller) = controllers.get(on_despawn.entity) else {
			return;
		};

		commands.try_apply_on(&controller.get_entity(), |e| e.try_despawn());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(MotionControllerOf::despawn_controlled);

		app
	}

	#[test]
	fn despawn() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.spawn(MotionControllerOf::skill(entity))
			.despawn();

		assert!(app.world().get_entity(entity).is_err());
	}
}
