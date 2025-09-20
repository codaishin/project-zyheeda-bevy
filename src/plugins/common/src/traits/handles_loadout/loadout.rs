use crate::{
	tools::skill_execution::SkillExecution,
	traits::{
		accessors::get::{GetProperty, Property},
		handles_localization::Token,
		thread_safe::ThreadSafe,
	},
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

pub trait LoadoutSkillItem:
	GetProperty<ItemToken>
	+ GetProperty<Result<SkillToken, NoSkill>>
	+ GetProperty<Result<SkillIcon, NoSkill>>
	+ GetProperty<Result<SkillExecution, NoSkill>>
{
}

impl<T> LoadoutSkillItem for T where
	T: GetProperty<ItemToken>
		+ GetProperty<Result<SkillToken, NoSkill>>
		+ GetProperty<Result<SkillIcon, NoSkill>>
		+ GetProperty<Result<SkillExecution, NoSkill>>
{
}

pub trait LoadoutSkill:
	PartialEq + Clone + ThreadSafe + GetProperty<SkillToken> + GetProperty<SkillIcon>
{
}

impl<T> LoadoutSkill for T where
	T: PartialEq + Clone + ThreadSafe + GetProperty<SkillToken> + GetProperty<SkillIcon>
{
}

pub struct ItemToken;

impl Property for ItemToken {
	type TValue<'a> = &'a Token;
}

pub struct SkillToken;

impl Property for SkillToken {
	type TValue<'a> = &'a Token;
}

pub struct SkillIcon;

impl Property for SkillIcon {
	type TValue<'a> = &'a Handle<Image>;
}

#[derive(Debug, PartialEq)]
pub struct NoSkill;
