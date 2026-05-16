pub(crate) mod dto;

use crate::{skills::shape::OnSkillStop, traits::spawn_skill::extension::SkillConfigData};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	tools::action_key::slot::SlotKey,
	traits::handles_skill_physics::{Effect, SkillMount, SkillShape},
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
	fn mount(&self, slot: SlotKey) -> SkillMount {
		match &self.shape {
			SkillShape::SphereAoE(_) | SkillShape::Shield(_) => SkillMount::Center,
			SkillShape::Projectile(_) | SkillShape::Beam(_) => SkillMount::slot(slot),
		}
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
