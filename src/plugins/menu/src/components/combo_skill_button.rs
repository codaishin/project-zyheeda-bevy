use super::combo_overview::ComboOverview;
use crate::{
	components::{combo_overview::ComboSkill, label::UILabel},
	traits::insert_ui_content::InsertUiContent,
};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::{
	tools::action_key::slot::SlotKey,
	traits::{handles_localization::Localize, thread_safe::ThreadSafe},
};
use std::{fmt::Debug, marker::PhantomData};

#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct DropdownTrigger;

#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct Vertical;

#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct Horizontal;

#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct DropdownItem<TLayout>(PhantomData<TLayout>);

#[derive(Component, Debug, PartialEq)]
#[require(Node)]
pub(crate) struct ComboSkillButton<T, TId>
where
	T: 'static,
	TId: Debug + PartialEq + Clone + ThreadSafe,
{
	_p: PhantomData<fn() -> T>,
	pub(crate) skill: ComboSkill<TId>,
	pub(crate) key_path: Vec<SlotKey>,
}

impl<T, TId> Clone for ComboSkillButton<T, TId>
where
	T: 'static,
	TId: Debug + PartialEq + Clone + ThreadSafe,
{
	fn clone(&self) -> Self {
		Self {
			_p: PhantomData,
			skill: self.skill.clone(),
			key_path: self.key_path.clone(),
		}
	}
}

impl<T, TId> ComboSkillButton<T, TId>
where
	T: 'static,
	TId: Debug + PartialEq + Clone + ThreadSafe,
{
	pub(crate) fn new(skill: ComboSkill<TId>, key_path: Vec<SlotKey>) -> Self {
		ComboSkillButton {
			_p: PhantomData,
			skill,
			key_path,
		}
	}
}

impl<T, TId> InsertUiContent for ComboSkillButton<T, TId>
where
	T: 'static,
	TId: Debug + PartialEq + Clone + ThreadSafe,
{
	fn insert_ui_content<TLocalization>(
		&self,
		_: &TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) where
		TLocalization: Localize,
	{
		parent.spawn((
			self.clone(),
			ComboOverview::skill_button(self.skill.icon.clone()),
			UILabel(self.skill.token.clone()),
		));
	}
}
