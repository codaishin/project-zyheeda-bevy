pub(crate) mod dto;

use super::SkillPrefab;
use crate::components::skill_prefabs::{FaultyColliderShape, skill_contact::dto::SkillContactDto};
use bevy::prelude::*;
use common::traits::{
	handles_skill_behaviors::{Contact, ContactShape, Motion},
	load_asset::LoadAsset,
	prefab::{Prefab, PrefabEntityCommands},
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

impl Prefab<()> for SkillContact {
	type TError = FaultyColliderShape;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), FaultyColliderShape> {
		let Ok(()) = self.motion.prefab(entity, self.created_from);
		self.shape.prefab(entity, Vec3::ZERO)
	}
}
