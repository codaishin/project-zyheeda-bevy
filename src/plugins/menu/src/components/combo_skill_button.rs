use super::combo_overview::ComboOverview;
use crate::{Tooltip, traits::insert_ui_content::InsertUiContent};
use bevy::prelude::*;
use common::{
	tools::{skill_description::SkillDescription, skill_icon::SkillIcon, slot_key::SlotKey},
	traits::{
		inspect_able::{InspectAble, InspectField},
		thread_safe::ThreadSafe,
	},
};
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
pub(crate) struct ComboSkillButton<T, TSkill> {
	phantom_data: PhantomData<T>,
	pub(crate) skill: TSkill,
	pub(crate) key_path: Vec<SlotKey>,
}

impl<T, TSkill> ComboSkillButton<T, TSkill> {
	pub(crate) fn new(skill: TSkill, key_path: Vec<SlotKey>) -> ComboSkillButton<T, TSkill> {
		ComboSkillButton {
			phantom_data: PhantomData,
			skill,
			key_path,
		}
	}
}

impl<T, TSkill> InsertUiContent for ComboSkillButton<T, TSkill>
where
	T: Clone + ThreadSafe,
	TSkill: InspectAble<SkillDescription> + InspectAble<SkillIcon> + Clone + ThreadSafe,
{
	fn insert_ui_content(&self, parent: &mut ChildBuilder) {
		parent.spawn((
			self.clone(),
			ComboOverview::skill_button(SkillIcon::inspect_field(&self.skill).clone()),
			Name::from(SkillDescription::inspect_field(&self.skill)),
			Tooltip::new(SkillDescription::inspect_field(&self.skill)),
		));
	}
}
