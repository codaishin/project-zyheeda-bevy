use crate::components::facing::{CanFace, SetFace, SetFaceOverride};
use bevy::prelude::*;
use common::traits::handles_orientation::Face;

impl SetFace {
	#[allow(clippy::type_complexity)]
	pub(crate) fn get_faces(
		faces: Query<(Entity, Option<&Self>, Option<&SetFaceOverride>), With<CanFace>>,
	) -> Vec<(Entity, Face)> {
		faces.iter().filter_map(face_value).collect()
	}
}

fn face_value(
	(id, set_face, override_face): (Entity, Option<&SetFace>, Option<&SetFaceOverride>),
) -> Option<(Entity, Face)> {
	match (set_face, override_face) {
		(Some(SetFace(face)), None) => Some((id, *face)),
		(_, Some(SetFaceOverride(face))) => Some((id, *face)),
		_ => None,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
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
		let agent = app.world_mut().spawn(SetFaceOverride(face)).id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Face(face)), agent.get::<_Face>());
	}

	#[test]
	fn prefer_faces_from_override_face() {
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

	#[test]
	fn ignore_when_can_face_component_missing() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn(SetFace(Face::Target))
			.remove::<CanFace>()
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_Face>());
	}
}
