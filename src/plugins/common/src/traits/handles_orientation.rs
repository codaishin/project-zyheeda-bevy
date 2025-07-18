use crate::components::persistent_entity::PersistentEntity;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub trait HandlesOrientation {
	type TFaceTemporarily: Component;

	fn temporarily(face: Face) -> Self::TFaceTemporarily;
}

#[derive(Default, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Face {
	#[default]
	Cursor,
	Entity(PersistentEntity),
	Translation(Vec3),
}
