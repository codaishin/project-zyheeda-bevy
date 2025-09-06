use crate::{
	tools::skill_execution::SkillExecution,
	traits::{accessors::get::RefInto, handles_localization::Token, thread_safe::ThreadSafe},
};
use bevy::prelude::*;

pub trait LoadoutKey {
	type TKey: Copy + ThreadSafe;
}

pub trait LoadoutItem {
	type TItem;
}

pub trait SwapInternal: LoadoutKey {
	fn swap_internal<TKey>(&mut self, a: TKey, b: TKey)
	where
		TKey: Into<Self::TKey> + 'static;
}

pub trait SwapExternal<TOther>: LoadoutKey
where
	TOther: LoadoutKey,
{
	fn swap_external<TKey, TOtherKey>(&mut self, other: &mut TOther, a: TKey, b: TOtherKey)
	where
		TKey: Into<Self::TKey> + 'static,
		TOtherKey: Into<TOther::TKey> + 'static;
}

pub trait LoadoutItemEntry:
	for<'a> RefInto<'a, Result<ItemToken<'a>, NoItem>>
	+ for<'a> RefInto<'a, Result<SkillToken<'a>, NoSkill>>
	+ for<'a> RefInto<'a, Result<SkillIcon<'a>, NoSkill>>
	+ for<'a> RefInto<'a, Result<&'a SkillExecution, NoSkill>>
{
}

impl<T> LoadoutItemEntry for T where
	T: for<'a> RefInto<'a, Result<ItemToken<'a>, NoItem>>
		+ for<'a> RefInto<'a, Result<SkillToken<'a>, NoSkill>>
		+ for<'a> RefInto<'a, Result<SkillIcon<'a>, NoSkill>>
		+ for<'a> RefInto<'a, Result<&'a SkillExecution, NoSkill>>
{
}

pub trait LoadoutSkill:
	PartialEq
	+ Clone
	+ ThreadSafe
	+ for<'a> RefInto<'a, SkillToken<'a>>
	+ for<'a> RefInto<'a, SkillIcon<'a>>
{
}

impl<T> LoadoutSkill for T where
	T: PartialEq
		+ Clone
		+ ThreadSafe
		+ for<'a> RefInto<'a, SkillToken<'a>>
		+ for<'a> RefInto<'a, SkillIcon<'a>>
{
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemToken<'a>(pub &'a Token);

#[derive(Debug, PartialEq, Clone)]
pub struct SkillToken<'a>(pub &'a Token);

#[derive(Debug, PartialEq, Clone)]
pub struct SkillIcon<'a>(pub &'a Handle<Image>);

#[derive(Debug, PartialEq)]
pub struct NoItem;

#[derive(Debug, PartialEq)]
pub struct NoSkill;
