use crate::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::{
		handles_input::InvalidUserInput,
		handles_localization::Token,
		iteration::{Iter, IterFinite},
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

impl From<TerrainTargeting> for ActionKey {
	fn from(target: TerrainTargeting) -> Self {
		Self::Targeting(target)
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
	fn iterator() -> Iter<Self> {
		Iter(Some(TerrainTargeting))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			TerrainTargeting => None,
		}
	}
}
