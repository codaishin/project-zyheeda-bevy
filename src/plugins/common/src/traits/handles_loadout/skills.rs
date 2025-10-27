use crate::{
	tools::skill_execution::SkillExecution,
	traits::{
		accessors::get::{GetProperty, Property},
		handles_loadout::LoadoutKey,
		handles_localization::Token,
	},
};
use bevy::prelude::*;
use std::ops::Deref;

pub struct Skills;

pub struct SkillToken;

impl Property for SkillToken {
	type TValue<'a> = &'a Token;
}

pub struct SkillIcon;

impl Property for SkillIcon {
	type TValue<'a> = &'a Handle<Image>;
}

pub trait GetSkillId<TSkillId> {
	fn get_skill_id(&self) -> TSkillId;
}

pub trait ReadSkills {
	type TSkill<'a>: GetProperty<SkillToken> + GetProperty<SkillIcon> + GetProperty<SkillExecution>
	where
		Self: 'a;

	fn get_skill<TKey>(&self, key: TKey) -> Option<Self::TSkill<'_>>
	where
		TKey: Into<LoadoutKey>;
}

impl<T> ReadSkills for T
where
	T: Deref<Target: ReadSkills>,
{
	type TSkill<'a>
		= <<T as Deref>::Target as ReadSkills>::TSkill<'a>
	where
		Self: 'a;

	fn get_skill<TKey>(&self, key: TKey) -> Option<Self::TSkill<'_>>
	where
		TKey: Into<LoadoutKey>,
	{
		self.deref().get_skill(key)
	}
}
