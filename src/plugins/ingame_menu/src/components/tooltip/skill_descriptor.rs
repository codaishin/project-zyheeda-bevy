use super::Tooltip;
use crate::{
	tools::{skill_name, skill_node, SkillDescriptor},
	traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn},
};
use bevy::prelude::{ChildBuilder, KeyCode, NodeBundle};

impl GetNode for Tooltip<SkillDescriptor<KeyCode>> {
	fn node(&self) -> NodeBundle {
		skill_node()
	}
}

impl InstantiateContentOn for Tooltip<SkillDescriptor<KeyCode>> {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		parent.spawn(skill_name(&self.0.skill.name));
	}
}
