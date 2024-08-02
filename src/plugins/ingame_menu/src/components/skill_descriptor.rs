use super::combo_overview::{ComboOverview, SkillButtonBundle};
use crate::traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn};
use bevy::prelude::{ChildBuilder, Component, NodeBundle};
use skills::{items::slot_key::SlotKey, skills::Skill};
use std::marker::PhantomData;

#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct DropdownTrigger;

#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct DropdownItem;

#[derive(Component, Debug, Default, PartialEq, Clone)]
pub(crate) struct SkillDescriptor<T = DropdownItem> {
	phantom_data: PhantomData<T>,
	pub(crate) skill: Skill,
	pub(crate) key_path: Vec<SlotKey>,
}

impl SkillDescriptor {
	pub(crate) fn new_dropdown_item(skill: Skill, key_path: Vec<SlotKey>) -> Self {
		Self {
			phantom_data: PhantomData,
			skill,
			key_path,
		}
	}

	#[cfg(test)]
	pub(crate) fn new_dropdown_trigger(
		skill: Skill,
		key_path: Vec<SlotKey>,
	) -> SkillDescriptor<DropdownTrigger> {
		SkillDescriptor {
			phantom_data: PhantomData,
			skill,
			key_path,
		}
	}

	pub(crate) fn to_dropdown_trigger(&self) -> SkillDescriptor<DropdownTrigger> {
		SkillDescriptor {
			phantom_data: PhantomData,
			skill: self.skill.clone(),
			key_path: self.key_path.clone(),
		}
	}
}

impl<T> GetNode for SkillDescriptor<T> {
	fn node(&self) -> NodeBundle {
		NodeBundle::default()
	}
}

impl InstantiateContentOn for SkillDescriptor {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		let icon = Some(self.skill.icon.clone().unwrap_or_default());
		parent.spawn(ComboOverview::skill_button_bundle(icon).with(self.clone()));
	}
}
