pub(crate) mod dto;

use super::SimplePrefab;
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{
	errors::Error,
	traits::{
		handles_destruction::HandlesDestruction,
		handles_interactions::HandlesInteractions,
		handles_skill_behaviors::{Contact, Integrity, Motion, Shape},
		prefab::Prefab,
	},
};

#[derive(Component, Debug, Clone)]
#[require(Visibility, Transform)]
pub struct SkillContact {
	pub(crate) created_from: CreatedFrom,
	pub(crate) shape: Shape,
	pub(crate) integrity: Integrity,
	pub(crate) motion: Motion,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum CreatedFrom {
	Contact,
	Save,
}

impl From<Contact> for SkillContact {
	fn from(
		Contact {
			shape,
			integrity,
			motion,
		}: Contact,
	) -> Self {
		Self {
			created_from: CreatedFrom::Contact,
			shape,
			integrity,
			motion,
		}
	}
}

impl<TInteractions, TLifeCycles> Prefab<(TInteractions, TLifeCycles)> for SkillContact
where
	TInteractions: HandlesInteractions,
	TLifeCycles: HandlesDestruction,
{
	fn insert_prefab_components(&self, entity: &mut EntityCommands) -> Result<(), Error> {
		self.shape
			.prefab::<TInteractions, TLifeCycles>(entity, Vec3::ZERO)?;
		self.motion
			.prefab::<TInteractions, TLifeCycles>(entity, self.created_from)?;
		self.integrity
			.prefab::<TInteractions, TLifeCycles>(entity, ())
	}
}
