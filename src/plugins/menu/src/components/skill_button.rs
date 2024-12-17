use super::{combo_overview::ComboOverview, tooltip::Tooltip};
use crate::traits::insert_ui_content::InsertUiContent;
use bevy::prelude::*;
use skills::{skills::Skill, slot_key::SlotKey};
use std::marker::PhantomData;

#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct DropdownTrigger;

#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct Vertical;

#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct Horizontal;

#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct DropdownItem<TLayout>(PhantomData<TLayout>);

#[derive(Component, Debug, Default, PartialEq, Clone)]
#[require(Node)]
pub(crate) struct SkillButton<T> {
	phantom_data: PhantomData<T>,
	pub(crate) skill: Skill,
	pub(crate) key_path: Vec<SlotKey>,
}

impl<T> SkillButton<T> {
	pub(crate) fn new(skill: Skill, key_path: Vec<SlotKey>) -> Self {
		Self {
			phantom_data: PhantomData,
			skill,
			key_path,
		}
	}
}

impl<T: Clone + Sync + Send + 'static> InsertUiContent for SkillButton<T> {
	fn insert_ui_content(&self, parent: &mut ChildBuilder) {
		parent.spawn((
			self.clone(),
			ComboOverview::skill_button(self.skill.icon.clone()),
			Tooltip::<Skill>::new(self.skill.clone()),
		));
	}
}
