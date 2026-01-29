use crate::components::world_camera::WorldCamera;
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl WorldCamera {
	pub(crate) fn remove_old_cameras(
		on_insert: On<Insert, Self>,
		mut commands: ZyheedaCommands,
		cameras: Query<Entity, With<Self>>,
	) {
		let entity = on_insert.entity;

		for cam in &cameras {
			if cam == entity {
				continue;
			}

			commands.try_apply_on(&cam, |mut e| {
				e.try_remove::<Self>();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(WorldCamera::remove_old_cameras);

		app
	}

	#[test]
	fn remove_previously_added_cameras() {
		let mut app = setup();

		let a = app.world_mut().spawn(WorldCamera::default()).id();
		let b = app.world_mut().spawn(WorldCamera::default()).id();

		assert_eq!(
			[None, Some(&WorldCamera::default())],
			app.world().entity([a, b]).map(|e| e.get::<WorldCamera>())
		);
	}
}
