use crate::components::{SetFaceOverride, SetFace};
use bevy::prelude::*;
use common::traits::handles_orientation::Face;

impl SetFace {
	pub(crate) fn get_faces(
		faces: Query<(Entity, &Self, Option<&SetFaceOverride>)>,
	) -> Vec<(Entity, Face)> {
		faces.iter().filter_map(face_value).collect()
	}
}

fn face_value(
	(id, SetFace(set_face), override_face): (Entity, &SetFace, Option<&SetFaceOverride>),
) -> Option<(Entity, Face)> {
	match override_face {
		Some(SetFaceOverride(override_face)) => Some((id, *override_face)),
		None => Some((id, *set_face)),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::SetFace;
	use bevy::{
		app::{App, Update},
		ecs::{
			component::Component,
			system::{Commands, In, IntoSystem},
		},
		math::Vec3,
	};
	use testing::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq)]
	struct _Face(Face);

	fn track_face(faces: In<Vec<(Entity, Face)>>, mut commands: Commands) {
		for (id, face) in faces.0 {
			commands.entity(id).insert(_Face(face));
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, SetFace::get_faces.pipe(track_face));

		app
	}

	#[test]
	fn get_faces_from_set_face() {
		let mut app = setup();
		let face = Face::Translation(Vec3::new(1., 2., 3.));
		let agent = app.world_mut().spawn(SetFace(face)).id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Face(face)), agent.get::<_Face>());
	}

	#[test]
	fn get_faces_from_override_face() {
		let mut app = setup();
		let face = Face::Translation(Vec3::new(1., 2., 3.));
		let agent = app
			.world_mut()
			.spawn((SetFace(Face::Target), SetFaceOverride(face)))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Face(face)), agent.get::<_Face>());
	}
}
