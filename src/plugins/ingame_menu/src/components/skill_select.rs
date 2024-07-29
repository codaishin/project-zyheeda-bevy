use bevy::prelude::{ChildBuilder, Component, NodeBundle};
use skills::{items::slot_key::SlotKey, skills::Skill};

use crate::traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn};

use super::{combo_overview::ComboOverview, tooltip::Tooltip};

#[derive(Component, Debug, Default, PartialEq, Clone)]
pub(crate) struct SkillSelect<TEquipmentKey = SlotKey> {
	pub(crate) skill: Skill,
	pub(crate) key_path: Vec<TEquipmentKey>,
}

impl GetNode for SkillSelect {
	fn node(&self) -> NodeBundle {
		NodeBundle::default()
	}
}

impl InstantiateContentOn for SkillSelect {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		parent.spawn((
			self.clone(),
			ComboOverview::skill_button_bundle(Some(self.skill.icon.clone().unwrap_or_default())),
			Tooltip::new(self.skill.clone()),
		));
	}
}
