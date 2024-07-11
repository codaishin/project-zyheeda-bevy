use crate::components::{Face, OverrideFace, SetFace};
use bevy::ecs::{entity::Entity, system::Query};

pub(crate) fn get_faces(
	faces: Query<(Entity, Option<&SetFace>, Option<&OverrideFace>)>,
) -> Vec<(Entity, Face)> {
	faces.iter().filter_map(face_value).collect()
}

fn face_value(
	(id, set_face, override_face): (Entity, Option<&SetFace>, Option<&OverrideFace>),
) -> Option<(Entity, Face)> {
	match (set_face, override_face) {
		(.., Some(override_face)) => Some((id, override_face.0)),
		(Some(set_face), None) => Some((id, set_face.0)),
		_ => None,
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
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq)]
	struct _Face(Face);

	fn track_face(faces: In<Vec<(Entity, Face)>>, mut commands: Commands) {
		for (id, face) in faces.0 {
			commands.entity(id).insert(_Face(face));
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, get_faces.pipe(track_face));

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
		let agent = app.world_mut().spawn(OverrideFace(face)).id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Face(face)), agent.get::<_Face>());
	}

	#[test]
	fn get_faces_from_override_face_even_when_set_face_set() {
		let mut app = setup();
		let face = Face::Translation(Vec3::new(1., 2., 3.));
		let agent = app
			.world_mut()
			.spawn((SetFace(Face::Cursor), OverrideFace(face)))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Face(face)), agent.get::<_Face>());
	}
}
