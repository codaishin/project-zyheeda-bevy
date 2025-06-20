pub(crate) mod dto;

use super::SimplePrefab;
use crate::components::skill_behavior::skill_contact::dto::SkillContactDto;
use bevy::prelude::*;
use common::{
	errors::Error,
	traits::{
		handles_destruction::HandlesDestruction,
		handles_interactions::HandlesInteractions,
		handles_saving::SavableComponent,
		handles_skill_behaviors::{Contact, Integrity, Motion, Shape},
		prefab::{Prefab, PrefabEntityCommands},
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
	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
	) -> Result<(), Error> {
		self.shape
			.prefab::<TInteractions, TLifeCycles>(entity, Vec3::ZERO)?;
		self.motion
			.prefab::<TInteractions, TLifeCycles>(entity, self.created_from)?;
		self.integrity
			.prefab::<TInteractions, TLifeCycles>(entity, ())
	}
}

impl SavableComponent for SkillContact {
	type TDto = SkillContactDto;
}
