use crate::{
	components::{SetFace, SetFaceOverride},
	system_param::face_param::FaceContextMut,
};
use common::traits::handles_orientation::{Face, OverrideFace};

impl OverrideFace for FaceContextMut<'_> {
	fn override_face(&mut self, face: Face) {
		self.entity
			.try_insert_if_new(SetFace(face))
			.try_insert(SetFaceOverride(face));
	}
	fn stop_override_face(&mut self) {}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{SetFace, SetFaceOverride},
		system_param::face_param::FaceParamMut,
	};
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
		math::Dir3,
	};
	use common::traits::{accessors::get::EntityContextMut, handles_orientation::Facing};
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
				let mut ctx = FaceParamMut::get_entity_context_mut(&mut p, entity, Facing).unwrap();
				ctx.override_face(Face::Target);
			})?;

		assert_eq!(
			(
				Some(&SetFace(Face::Target)),
				Some(&SetFaceOverride(Face::Target)),
			),
			(
				app.world().entity(entity).get::<SetFace>(),
				app.world().entity(entity).get::<SetFaceOverride>(),
			)
		);
		Ok(())
	}

	#[test]
	fn leave_set_face_untouched() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(SetFace(Face::Direction(Dir3::NEG_X)))
			.id();

		app.world_mut()
			.run_system_once(move |mut p: FaceParamMut| {
				let mut ctx = FaceParamMut::get_entity_context_mut(&mut p, entity, Facing).unwrap();
				ctx.override_face(Face::Target);
			})?;

		assert_eq!(
			(
				Some(&SetFace(Face::Direction(Dir3::NEG_X))),
				Some(&SetFaceOverride(Face::Target)),
			),
			(
				app.world().entity(entity).get::<SetFace>(),
				app.world().entity(entity).get::<SetFaceOverride>(),
			)
		);
		Ok(())
	}
}
