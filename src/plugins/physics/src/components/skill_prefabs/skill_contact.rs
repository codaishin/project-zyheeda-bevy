pub(crate) mod dto;

use super::SkillPrefab;
use crate::components::skill_prefabs::skill_contact::dto::SkillContactDto;
use bevy::prelude::*;
use common::{
	errors::Unreachable,
	traits::{
		handles_skill_physics::{Contact, ContactShape, Motion},
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use macros::SavableComponent;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[require(Visibility, Transform)]
#[savable_component(dto = SkillContactDto)]
pub struct SkillContact {
	pub(crate) created_from: CreatedFrom,
	pub(crate) shape: ContactShape,
	pub(crate) motion: Motion,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum CreatedFrom {
	Spawn,
	Save,
}

impl From<Contact> for SkillContact {
	fn from(Contact { shape, motion }: Contact) -> Self {
		Self {
			created_from: CreatedFrom::Spawn,
			shape,
			motion,
		}
	}
}

impl Prefab<()> for SkillContact {
	type TError = Unreachable;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Unreachable> {
		let Ok(()) = self.motion.prefab(entity, self.created_from);
		self.shape.prefab(entity, Vec3::ZERO)
	}
}
