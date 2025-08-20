use crate::components::{OverrideFace, SetFace};
use bevy::prelude::*;
use common::traits::handles_orientation::Face;

impl<T> GetFaces for T where T: Component {}

pub(crate) trait GetFaces: Component + Sized {
	#[allow(clippy::type_complexity)]
	fn get_faces(
		faces: Query<(Entity, Option<&SetFace>, Option<&OverrideFace>), With<Self>>,
	) -> Vec<(Entity, Face)> {
		faces.iter().filter_map(face_value).collect()
	}
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
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, Debug, PartialEq)]
	struct _Face(Face);

	fn track_face(faces: In<Vec<(Entity, Face)>>, mut commands: Commands) {
		for (id, face) in faces.0 {
			commands.entity(id).insert(_Face(face));
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _Agent::get_faces.pipe(track_face));

		app
	}

	#[test]
	fn get_faces_from_set_face() {
		let mut app = setup();
		let face = Face::Translation(Vec3::new(1., 2., 3.));
		let agent = app.world_mut().spawn((_Agent, SetFace(face))).id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Face(face)), agent.get::<_Face>());
	}

	#[test]
	fn get_faces_from_override_face() {
		let mut app = setup();
		let face = Face::Translation(Vec3::new(1., 2., 3.));
		let agent = app.world_mut().spawn((_Agent, OverrideFace(face))).id();

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
			.spawn((_Agent, SetFace(Face::Target), OverrideFace(face)))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Face(face)), agent.get::<_Face>());
	}

	#[test]
	fn ignore_when_no_agent() {
		let mut app = setup();
		let face = Face::Translation(Vec3::new(1., 2., 3.));
		let agent = app.world_mut().spawn(SetFace(face)).id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_Face>());
	}
}
