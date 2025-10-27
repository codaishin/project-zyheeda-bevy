use crate::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::GetProperty,
		handles_loadout::skills::{GetSkillId, SkillIcon, SkillToken},
	},
};
use std::ops::Deref;

pub struct AvailableSkills;

pub trait ReadAvailableSkills<TSkillID> {
	type TSkill<'a>: GetProperty<SkillToken> + GetProperty<SkillIcon> + GetSkillId<TSkillID>
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
