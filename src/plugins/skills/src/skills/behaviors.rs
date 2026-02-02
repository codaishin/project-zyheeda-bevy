pub(crate) mod dto;

use bevy::prelude::*;
use common::traits::handles_skill_physics::{Effect, SkillShape};

#[derive(PartialEq, Debug, Clone)]
pub struct SkillBehaviorConfig {
	pub(crate) shape: SkillShape,
	pub(crate) contact: Vec<Effect>,
	pub(crate) projection: Vec<Effect>,
}

impl SkillBehaviorConfig {
	#[cfg(test)]
	pub(crate) const fn from_shape(shape: SkillShape) -> Self {
		Self {
			shape,
			contact: vec![],
			projection: vec![],
		}
	}

	#[cfg(test)]
	pub(crate) fn with_contact_effects(self, contact: Vec<Effect>) -> Self {
		Self {
			shape: self.shape,
			contact,
			projection: self.projection,
		}
	}

	#[cfg(test)]
	pub(crate) fn with_projection_effects(self, projection: Vec<Effect>) -> Self {
		Self {
			shape: self.shape,
			contact: self.contact,
			projection,
		}
	}
}
