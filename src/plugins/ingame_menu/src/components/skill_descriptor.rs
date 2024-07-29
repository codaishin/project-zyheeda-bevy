use super::combo_overview::{ComboOverview, SkillButtonBundle};
use crate::traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn};
use bevy::prelude::{ChildBuilder, Component, NodeBundle};
use skills::{items::slot_key::SlotKey, skills::Skill};

#[derive(Component, Debug, Default, PartialEq, Clone)]
pub(crate) struct SkillDescriptor<TEquipmentKey = SlotKey> {
	pub(crate) skill: Skill,
	pub(crate) key_path: Vec<TEquipmentKey>,
}

impl GetNode for SkillDescriptor {
	fn node(&self) -> NodeBundle {
		NodeBundle::default()
	}
}

impl InstantiateContentOn for SkillDescriptor {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		let icon = Some(self.skill.icon.clone().unwrap_or_default());
		parent.spawn(ComboOverview::skill_button_bundle(icon).with(self));
	}
}
