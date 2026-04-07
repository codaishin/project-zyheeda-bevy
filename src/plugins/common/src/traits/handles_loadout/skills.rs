use crate::{
	tools::skill_execution::SkillExecution,
	traits::{
		accessors::get::{View, ViewField},
		handles_loadout::LoadoutKey,
		handles_localization::Token,
	},
};
use bevy::prelude::*;
use macros::EntityKey;
use std::ops::Deref;

#[derive(EntityKey)]
pub struct Skills {
	pub entity: Entity,
}

pub struct SkillToken;

impl ViewField for SkillToken {
	type TValue<'a> = &'a Token;
}

pub struct SkillIcon;

impl ViewField for SkillIcon {
	type TValue<'a> = &'a Handle<Image>;
}

pub trait GetSkillId<TSkillId> {
	fn get_skill_id(&self) -> TSkillId;
}

pub trait ReadSkills {
	type TSkill<'a>: View<SkillToken> + View<SkillIcon> + View<SkillExecution>
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
