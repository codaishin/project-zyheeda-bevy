use super::Tooltip;
use crate::{
	components::skill_select::SkillSelect,
	tools::{skill_name, skill_node},
	traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn},
};
use bevy::prelude::{ChildBuilder, NodeBundle};

impl GetNode for Tooltip<SkillSelect> {
	fn node(&self) -> NodeBundle {
		skill_node()
	}
}

impl InstantiateContentOn for Tooltip<SkillSelect> {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		parent.spawn(skill_name(&self.0.skill.name));
	}
}
