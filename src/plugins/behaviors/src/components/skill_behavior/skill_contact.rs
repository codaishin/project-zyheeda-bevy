pub(crate) mod dto;

use super::SimplePrefab;
use crate::components::skill_behavior::skill_contact::dto::SkillContactDto;
use bevy::prelude::*;
use common::{
	errors::Error,
	traits::{
		handles_interactions::HandlesInteractions,
		handles_skill_behaviors::{Contact, Motion, ContactShape},
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use macros::SavableComponent;

#[derive(Component, SavableComponent, Debug, Clone)]
#[require(Visibility, Transform)]
#[savable_component(dto = SkillContactDto)]
pub struct SkillContact {
	pub(crate) created_from: CreatedFrom,
	pub(crate) shape: ContactShape,
	pub(crate) motion: Motion,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum CreatedFrom {
	Contact,
	Save,
}

impl From<Contact> for SkillContact {
	fn from(Contact { shape, motion }: Contact) -> Self {
		Self {
			created_from: CreatedFrom::Contact,
			shape,
			motion,
		}
	}
}

impl<TInteractions> Prefab<TInteractions> for SkillContact
where
	TInteractions: HandlesInteractions,
{
	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Error> {
		let created_from = self.created_from;

		self.shape.prefab::<TInteractions>(entity, Vec3::ZERO)?;
		self.motion.prefab::<TInteractions>(entity, created_from)
	}
}
