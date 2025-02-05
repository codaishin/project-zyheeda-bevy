use super::{combo_overview::ComboOverview, tooltip::Tooltip};
use crate::traits::insert_ui_content::InsertUiContent;
use bevy::prelude::*;
use common::{
	tools::slot_key::SlotKey,
	traits::{
		accessors::get::{GetField, GetFieldRef, Getter, GetterRef},
		handles_equipment::SkillDescription,
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
pub(crate) struct SkillButton<T, TSkill> {
	phantom_data: PhantomData<T>,
	pub(crate) skill: TSkill,
	pub(crate) key_path: Vec<SlotKey>,
}

impl<T, TSkill> SkillButton<T, TSkill> {
	pub(crate) fn new(skill: TSkill, key_path: Vec<SlotKey>) -> SkillButton<T, TSkill> {
		SkillButton {
			phantom_data: PhantomData,
			skill,
			key_path,
		}
	}
}

impl<T, TSkill> InsertUiContent for SkillButton<T, TSkill>
where
	T: Clone + ThreadSafe,
	TSkill: GetterRef<Option<Handle<Image>>> + Getter<SkillDescription> + Clone + ThreadSafe,
{
	fn insert_ui_content(&self, parent: &mut ChildBuilder) {
		let icon = Option::<Handle<Image>>::get_field_ref(&self.skill).clone();
		let description = SkillDescription::get_field(&self.skill);

		parent.spawn((
			self.clone(),
			ComboOverview::skill_button(icon),
			Tooltip::new(description),
		));
	}
}
