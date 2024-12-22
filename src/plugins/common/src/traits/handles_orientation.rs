use bevy::prelude::*;

pub trait HandlesOrientation {
	type TFaceTemporarily: Component;

	fn temporarily(face: Face) -> Self::TFaceTemporarily;
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum Face {
	#[default]
	Cursor,
	Entity(Entity),
	Translation(Vec3),
}
