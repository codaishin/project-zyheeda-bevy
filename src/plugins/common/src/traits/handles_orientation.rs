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
	/// This is dependent on the type of agent, for a player it likely means the cursor.
	Target,
	Entity(PersistentEntity),
	Translation(Vec3),
	Direction(Dir3),
}
