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
	for<'a> GetProperty<ItemToken<'a>>
	+ for<'a> GetProperty<Result<SkillToken<'a>, NoSkill>>
	+ for<'a> GetProperty<Result<SkillIcon<'a>, NoSkill>>
	+ GetProperty<Result<SkillExecution, NoSkill>>
{
}

impl<T> LoadoutSkillItem for T where
	T: for<'a> GetProperty<ItemToken<'a>>
		+ for<'a> GetProperty<Result<SkillToken<'a>, NoSkill>>
		+ for<'a> GetProperty<Result<SkillIcon<'a>, NoSkill>>
		+ GetProperty<Result<SkillExecution, NoSkill>>
{
}

pub trait LoadoutSkill:
	PartialEq
	+ Clone
	+ ThreadSafe
	+ for<'a> GetProperty<SkillToken<'a>>
	+ for<'a> GetProperty<SkillIcon<'a>>
{
}

impl<T> LoadoutSkill for T where
	T: PartialEq
		+ Clone
		+ ThreadSafe
		+ for<'a> GetProperty<SkillToken<'a>>
		+ for<'a> GetProperty<SkillIcon<'a>>
{
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemToken<'a>(pub &'a Token);

impl Property for ItemToken<'_> {
	type TValue<'a> = &'a Token;
}

#[derive(Debug, PartialEq, Clone)]
pub struct SkillToken<'a>(pub &'a Token);

impl Property for SkillToken<'_> {
	type TValue<'a> = &'a Token;
}

#[derive(Debug, PartialEq, Clone)]
pub struct SkillIcon<'a>(pub &'a Handle<Image>);

impl Property for SkillIcon<'_> {
	type TValue<'a> = &'a Handle<Image>;
}

#[derive(Debug, PartialEq)]
pub struct NoSkill;
