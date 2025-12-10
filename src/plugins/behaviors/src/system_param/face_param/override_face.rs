use crate::{components::facing::SetFaceOverride, system_param::face_param::FaceContextMut};
use common::traits::handles_orientation::{Face, OverrideFace};

impl OverrideFace for FaceContextMut<'_> {
	fn override_face(&mut self, face: Face) {
		self.entity.try_insert(SetFaceOverride(face));
	}
	fn stop_override_face(&mut self) {
		self.entity.try_remove::<SetFaceOverride>();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::system_param::face_param::FaceParamMut;
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::traits::{accessors::get::GetContextMut, handles_orientation::Facing};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_face_components() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: FaceParamMut| {
				let mut ctx = FaceParamMut::get_context_mut(&mut p, Facing { entity }).unwrap();
				ctx.override_face(Face::Target);
			})?;

		assert_eq!(
			Some(&SetFaceOverride(Face::Target)),
			app.world().entity(entity).get::<SetFaceOverride>(),
		);
		Ok(())
	}

	#[test]
	fn remove_face_components() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(SetFaceOverride(Face::Target)).id();

		app.world_mut()
			.run_system_once(move |mut p: FaceParamMut| {
				let mut ctx = FaceParamMut::get_context_mut(&mut p, Facing { entity }).unwrap();
				ctx.stop_override_face();
			})?;

		assert_eq!(None, app.world().entity(entity).get::<SetFaceOverride>(),);
		Ok(())
	}
}
