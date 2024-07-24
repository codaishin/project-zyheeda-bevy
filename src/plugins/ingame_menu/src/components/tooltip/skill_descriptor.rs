use super::Tooltip;
use crate::{
	tools::{skill_name, skill_node, SkillDescriptor},
	traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn},
};
use bevy::prelude::{ChildBuilder, KeyCode, NodeBundle};

impl<T: Clone> GetNode for Tooltip<SkillDescriptor<KeyCode, T>> {
	fn node(&self) -> NodeBundle {
		skill_node()
	}
}

impl<T: Clone> InstantiateContentOn for Tooltip<SkillDescriptor<KeyCode, T>> {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		parent.spawn(skill_name(&self.0.name));
	}
}
