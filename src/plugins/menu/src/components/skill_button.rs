use super::{combo_overview::ComboOverview, tooltip::Tooltip};
use crate::traits::{ui_components::GetUIComponents, update_children::UpdateChildren};
use bevy::{color::palettes::tailwind::CYAN_100, prelude::*};
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

impl<T> GetUIComponents for SkillButton<T> {
	fn ui_components(&self) -> (Node, BackgroundColor) {
		(default(), BackgroundColor(CYAN_100.into()))
	}
}

impl<T: Clone + Sync + Send + 'static> UpdateChildren for SkillButton<T> {
	fn update_children(&self, parent: &mut ChildBuilder) {
		let icon = self.skill.icon.clone();
		parent.spawn((
			self.clone(),
			ComboOverview::skill_button(icon),
			Tooltip::<Skill>::new(self.skill.clone()),
		));
	}
}
