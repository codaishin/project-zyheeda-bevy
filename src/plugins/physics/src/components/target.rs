use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{
		handles_animations::DirForwardPitch,
		handles_skill_physics::{Cursor, SkillTarget},
	},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

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

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct OldTargetPitch(pub(crate) Option<DirForwardPitch>);

impl Deref for OldTargetPitch {
	type Target = Option<DirForwardPitch>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::ApproxEqual;

	impl ApproxEqual<f32> for OldTargetPitch {
		fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
			match (self.0, other.0) {
				(None, None) => true,
				(Some(DirForwardPitch::Down(l)), Some(DirForwardPitch::Down(r))) => {
					l.approx_equal(&r, tolerance)
				}
				(Some(DirForwardPitch::Up(l)), Some(DirForwardPitch::Up(r))) => {
					l.approx_equal(&r, tolerance)
				}
				_ => false,
			}
		}
	}
}
