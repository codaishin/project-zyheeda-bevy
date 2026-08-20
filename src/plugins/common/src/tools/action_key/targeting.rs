use crate::{
	tools::action_key::user_input::UserInput,
	traits::{
		handles_input::InvalidUserInput,
		handles_localization::Token,
		iteration::{FiniteIter, IterFinite},
	},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(TypePath, Clone, Copy, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub struct TerrainTargeting;

impl InvalidUserInput for TerrainTargeting {
	fn invalid_input(&self) -> &[UserInput] {
		&[]
	}
}

impl From<TerrainTargeting> for UserInput {
	fn from(_: TerrainTargeting) -> Self {
		Self::KeyCode(KeyCode::ShiftLeft)
	}
}

impl From<TerrainTargeting> for Token {
	fn from(_: TerrainTargeting) -> Self {
		Self::from("terrain-targeting")
	}
}

impl IterFinite for TerrainTargeting {
	fn iterator() -> FiniteIter<Self> {
		FiniteIter(Some(TerrainTargeting))
	}

	fn next(current: &FiniteIter<Self>) -> Option<Self> {
		match current.0? {
			TerrainTargeting => None,
		}
	}
}
