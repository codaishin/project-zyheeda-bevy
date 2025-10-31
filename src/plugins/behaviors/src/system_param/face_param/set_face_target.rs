use crate::{components::face_target::FaceTarget, system_param::face_param::FaceContextMut};
use common::traits::handles_orientation::{FaceTargetIs, SetFaceTarget};

impl SetFaceTarget for FaceContextMut<'_> {
	fn set_face_target(&mut self, target: FaceTargetIs) {
		self.entity.try_insert(FaceTarget(target));
	}
}

#[cfg(test)]
mod tests {
	use crate::{components::face_target::FaceTarget, system_param::face_param::FaceParamMut};

	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::traits::{accessors::get::EntityContextMut, handles_orientation::Facing};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_face_target() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: FaceParamMut| {
				let mut ctx = FaceParamMut::get_entity_context_mut(&mut p, entity, Facing).unwrap();
				ctx.set_face_target(FaceTargetIs::Cursor);
			})?;

		assert_eq!(
			Some(&FaceTarget(FaceTargetIs::Cursor)),
			app.world().entity(entity).get::<FaceTarget>()
		);
		Ok(())
	}
}
