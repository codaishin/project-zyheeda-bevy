use crate::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::View,
		handles_loadout::skills::{GetSkillId, SkillIcon, SkillToken},
	},
};
use bevy::prelude::*;
use macros::EntityKey;
use std::ops::Deref;

#[derive(EntityKey)]
pub struct AvailableSkills {
	pub entity: Entity,
}

pub trait ReadAvailableSkills<TSkillID> {
	type TSkill<'a>: View<SkillToken> + View<SkillIcon> + GetSkillId<TSkillID>
	where
		Self: 'a;

	fn get_available_skills(&self, key: SlotKey) -> impl Iterator<Item = Self::TSkill<'_>>;
}

impl<T, TSkillID> ReadAvailableSkills<TSkillID> for T
where
	T: Deref<Target: ReadAvailableSkills<TSkillID>>,
{
	type TSkill<'a>
		= <<T as Deref>::Target as ReadAvailableSkills<TSkillID>>::TSkill<'a>
	where
		Self: 'a;

	fn get_available_skills(&self, key: SlotKey) -> impl Iterator<Item = Self::TSkill<'_>> {
		self.deref().get_available_skills(key)
	}
}
