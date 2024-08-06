use super::combo_overview::{ComboOverview, SkillButtonBundle};
use crate::traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn};
use bevy::prelude::{ChildBuilder, Component, NodeBundle};
use skills::{items::slot_key::SlotKey, skills::Skill};
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
pub(crate) struct SkillDescriptor<T> {
	phantom_data: PhantomData<T>,
	pub(crate) skill: Skill,
	pub(crate) key_path: Vec<SlotKey>,
}

impl SkillDescriptor<DropdownTrigger> {
	pub(crate) fn new(skill: Skill, key_path: Vec<SlotKey>) -> SkillDescriptor<DropdownTrigger> {
		SkillDescriptor {
			phantom_data: PhantomData,
			skill,
			key_path,
		}
	}
}

impl<TLayout> SkillDescriptor<DropdownItem<TLayout>> {
	pub(crate) fn new(skill: Skill, key_path: Vec<SlotKey>) -> Self {
		Self {
			phantom_data: PhantomData,
			skill,
			key_path,
		}
	}
}

impl<T> GetNode for SkillDescriptor<T> {
	fn node(&self) -> NodeBundle {
		NodeBundle::default()
	}
}

impl<T: Clone + Sync + Send + 'static> InstantiateContentOn for SkillDescriptor<T> {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		let icon = self.skill.icon.clone().unwrap_or_default();
		parent.spawn(ComboOverview::skill_button_bundle(icon).with(self.clone()));
	}
}
