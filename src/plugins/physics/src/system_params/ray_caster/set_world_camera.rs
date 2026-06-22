use crate::{components::world_camera::WorldCamera, system_params::ray_caster::RayCasterMut};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::TryApplyOn,
		handles_physics::{NoWorldCameraSet, SetWorldCamera},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<'w, 's> NoWorldCameraSet for RayCasterMut<'w, 's> {
	type TSetter = WorldCameraSetter<'w, 's>;

	fn no_world_camera_set(self) -> Option<Self::TSetter> {
		if !self.inner.world_cameras.is_empty() {
			return None;
		}

		Some(WorldCameraSetter(self.inner.commands))
	}
}

pub struct WorldCameraSetter<'w, 's>(ZyheedaCommands<'w, 's>);

impl SetWorldCamera for WorldCameraSetter<'_, '_> {
	fn set_world_camera(mut self, entity: Entity) {
		self.0.try_apply_on(&entity, |mut e| {
			e.try_insert(WorldCamera::default());
		});
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{components::world_camera::WorldCamera, system_params::ray_caster::RayCasterMut};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn set_world_camera() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut().run_system_once(move |r: RayCasterMut| {
			let Some(setter) = r.no_world_camera_set() else {
				return;
			};

			setter.set_world_camera(entity);
		})?;

		assert_eq!(
			Some(&WorldCamera::default()),
			app.world().entity(entity).get::<WorldCamera>(),
		);
		Ok(())
	}

	#[test]
	fn do_not_set_world_camera_when_already_set() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(WorldCamera::default());

		let setter = app
			.world_mut()
			.run_system_once(move |r: RayCasterMut| r.no_world_camera_set().is_some())?;

		assert!(!setter);
		Ok(())
	}
}
