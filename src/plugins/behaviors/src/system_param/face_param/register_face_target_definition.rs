use crate::{components::face_target::FaceTarget, system_param::face_param::FaceContextMut};
use common::traits::handles_orientation::{FaceTargetIs, RegisterFaceTargetDefinition};

impl RegisterFaceTargetDefinition for FaceContextMut<'_> {
	fn register(&mut self, face_target: FaceTargetIs) {
		self.entity.try_insert(FaceTarget(face_target));
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{components::face_target::FaceTarget, system_param::face_param::FaceParamMut};
	use bevy::{
		app::App,
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{accessors::get::GetContextMut, handles_orientation::Facing};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn set_face_target() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut f: FaceParamMut| {
				let mut ctx = FaceParamMut::get_context_mut(&mut f, Facing { entity }).unwrap();
				ctx.register(FaceTargetIs::Cursor);
			})?;

		assert_eq!(
			Some(&FaceTarget(FaceTargetIs::Cursor)),
			app.world().entity(entity).get::<FaceTarget>()
		);
		Ok(())
	}
}
