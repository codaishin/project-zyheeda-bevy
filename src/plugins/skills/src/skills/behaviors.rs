pub(crate) mod dto;

use crate::{skills::shape::OnSkillStop, traits::spawn_skill::extension::SkillConfigData};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::handles_skill_physics::{Effect, SkillShape},
};

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
}

impl SkillConfigData for SkillBehaviorConfig {
	fn use_neutral_mount(&self) -> bool {
		matches!(
			&self.shape,
			SkillShape::Shield(_) | SkillShape::SphereAoE(_)
		)
	}

	fn shape(&self) -> &'_ SkillShape {
		&self.shape
	}

	fn contact_effects(&self) -> &'_ [Effect] {
		&self.contact
	}

	fn projection_effects(&self) -> &'_ [Effect] {
		&self.projection
	}

	fn on_skill_stop(&self, skill: PersistentEntity) -> OnSkillStop {
		match &self.shape {
			SkillShape::Beam(_) | SkillShape::Shield(_) => OnSkillStop::Stop(skill),
			_ => OnSkillStop::Ignore,
		}
	}
}
