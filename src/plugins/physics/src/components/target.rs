use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::handles_skill_physics::{Cursor, SkillTarget},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(SavableComponent, Component, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[savable_component(id = "skill target")]
pub(crate) struct Target(pub(crate) Option<SkillTarget>);

impl From<PersistentEntity> for Target {
	fn from(entity: PersistentEntity) -> Self {
		Self(Some(SkillTarget::Entity(entity)))
	}
}

impl From<Cursor> for Target {
	fn from(cursor: Cursor) -> Self {
		Self(Some(SkillTarget::Cursor(cursor)))
	}
}
